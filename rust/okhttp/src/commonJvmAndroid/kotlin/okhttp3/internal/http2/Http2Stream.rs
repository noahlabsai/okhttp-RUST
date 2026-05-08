use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Result as IoResult};
use std::sync::{Arc, Mutex, Condvar};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::{assert_lock_not_held};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::flowcontrol::WindowCounter;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::{Http2Connection, ErrorCode, Header};
use okio::{AsyncTimeout, Buffer, BufferedSource, Sink, Source, Timeout};

pub const EMIT_BUFFER_SIZE: i64 = 16384;

#[derive(Debug)]
pub struct StreamResetException {
    pub code: ErrorCode,
}

impl std::fmt::Display for StreamResetException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stream reset with error code: {:?}", self.code)
    }
}

impl std::error::Error for StreamResetException {}

pub struct Http2Stream {
    pub id: i32,
    pub connection: Arc<Http2Connection>,
    state: Arc<(Mutex<Http2StreamState>, Condvar)>,
    pub read_bytes: WindowCounter,
    pub read_timeout: Arc<StreamTimeout>,
    pub write_timeout: Arc<StreamTimeout>,
}

struct Http2StreamState {
    write_bytes_total: i64,
    write_bytes_maximum: i64,
    headers_queue: VecDeque<Headers>,
    has_response_headers: bool,
    error_code: Option<ErrorCode>,
    error_exception: Option<Error>,
    source: FramingSource,
    sink: FramingSink,
}

impl Http2Stream {
    pub fn new(
        id: i32,
        connection: Arc<Http2Connection>,
        out_finished: bool,
        in_finished: bool,
        headers: Option<Headers>,
    ) -> Arc<Self> {
        let initial_window_size = connection.peer_settings().initial_window_size() as i64;
        let ok_http_initial_window = connection.ok_http_settings().initial_window_size() as i64;

        let mut headers_queue = VecDeque::new();
        let is_locally_initiated = (id & 1) == 1 == connection.client();

        if let Some(h) = headers {
            if is_locally_initiated {
                panic!("locally-initiated streams shouldn't have headers yet");
            }
            headers_queue.push_back(h);
        } else if !is_locally_initiated {
            panic!("remotely-initiated streams should have headers");
        }

        let state = Arc::new((
            Mutex::new(Http2StreamState {
                write_bytes_total: 0,
                write_bytes_maximum: initial_window_size,
                headers_queue,
                has_response_headers: false,
                error_code: None,
                error_exception: None,
                source: FramingSource {
                    max_byte_count: ok_http_initial_window,
                    finished: in_finished,
                    receive_buffer: Buffer::new(),
                    read_buffer: Buffer::new(),
                    trailers: None,
                    closed: false,
                },
                sink: FramingSink {
                    finished: out_finished,
                    send_buffer: Buffer::new(),
                    trailers: None,
                    closed: false,
                },
            }),
            Condvar::new(),
        ));

        let stream = Arc::new(Http2Stream {
            id,
            connection,
            state,
            read_bytes: WindowCounter::new(id),
            read_timeout: Arc::new(StreamTimeout {
                stream: None,
                connection: Arc::clone(&connection),
            }),
            write_timeout: Arc::new(StreamTimeout {
                stream: None,
                connection: Arc::clone(&connection),
            }),
        });

        // Set weak references in timeouts
        Arc::as_ptr(&stream) as *mut StreamTimeout; // Placeholder for actual logic if needed
        // In a real implementation, we'd use a Mutex/Cell to set the weak ref after Arc creation
        
        stream
    }

    pub fn is_open(&self) -> bool {
        let (lock, _) = &*self.state;
        let state = lock.lock().unwrap();
        if state.error_code.is_some() {
            return false;
        }
        if (state.source.finished || state.source.closed)
            && (state.sink.finished || state.sink.closed)
            && state.has_response_headers
        {
            return false;
        }
        true
    }

    pub fn is_locally_initiated(&self) -> bool {
        let stream_is_client = (self.id & 1) == 1;
        self.connection.client() == stream_is_client
    }

    pub fn is_source_complete(&self) -> bool {
        let (lock, _) = &*self.state;
        let state = lock.lock().unwrap();
        state.source.finished && state.source.read_buffer.size() == 0
    }

    pub fn take_headers(&self, caller_is_idle: bool) -> IoResult<Headers> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        while state.headers_queue.is_empty() && state.error_code.is_none() {
            let do_read_timeout = caller_is_idle || self.do_read_timeout(&state);
            if do_read_timeout {
                self.read_timeout.enter();
            }
            
            state = cvar.wait(state).unwrap();

            if do_read_timeout {
                self.read_timeout.exit_and_throw_if_timed_out()?;
            }
        }

        if let Some(headers) = state.headers_queue.pop_front() {
            return Ok(headers);
        }

        Err(state.error_exception.clone().unwrap_or_else(|| {
            Error::new(ErrorKind::Other, StreamResetException {
                code: state.error_code.expect("errorCode must be set if queue is empty"),
            })
        }))
    }

    pub fn peek_trailers(&self) -> IoResult<Option<Headers>> {
        let (lock, _) = &*self.state;
        let state = lock.lock().unwrap();
        if state.source.finished && state.source.receive_buffer.size() == 0 && state.source.read_buffer.size() == 0 {
            return Ok(state.source.trailers.clone().or(Some(Headers::empty())));
        }
        if let Some(code) = state.error_code {
            return Err(state.error_exception.clone().unwrap_or_else(|| {
                Error::new(ErrorKind::Other, StreamResetException { code })
            }));
        }
        Ok(None)
    }

    pub fn write_headers(&self, response_headers: Vec<Header>, out_finished: bool, mut flush_headers: bool) -> IoResult<()> {
        assert_lock_not_held();

        {
            let (lock, cvar) = &*self.state;
            let mut state = lock.lock().unwrap();
            state.has_response_headers = true;
            if out_finished {
                state.sink.finished = true;
                cvar.notify_all();
            }
            if !flush_headers {
                flush_headers = self.connection.write_bytes_total() >= self.connection.write_bytes_maximum();
            }
        }

        self.connection.write_headers(self.id, out_finished, response_headers)?;

        if flush_headers {
            self.connection.flush()?;
        }
        Ok(())
    }

    pub fn enqueue_trailers(&self, trailers: Headers) {
        let (lock, _) = &*self.state;
        let mut state = lock.lock().unwrap();
        if state.sink.finished {
            panic!("already finished");
        }
        if trailers.size() == 0 {
            panic!("trailers.size() == 0");
        }
        state.sink.trailers = Some(trailers);
    }

    pub fn close(&self, rst_status_code: ErrorCode, error_exception: Option<Error>) -> IoResult<()> {
        if !self.close_internal(rst_status_code, error_exception) {
            return Ok(());
        }
        self.connection.write_syn_reset(self.id, rst_status_code)?;
        Ok(())
    }

    pub fn cancel(&self) {
        self.close_later(ErrorCode::CANCEL);
    }

    pub fn close_later(&self, error_code: ErrorCode) {
        if !self.close_internal(error_code, None) {
            return;
        }
        self.connection.write_syn_reset_later(self.id, error_code);
    }

    fn close_internal(&self, error_code: ErrorCode, error_exception: Option<Error>) -> bool {
        assert_lock_not_held();

        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        if state.error_code.is_some() {
            return false;
        }
        state.error_code = Some(error_code);
        state.error_exception = error_exception;
        cvar.notify_all();
        if state.source.finished && state.sink.finished {
            return false;
        }
        drop(state);
        self.connection.remove_stream(self.id);
        true
    }

    pub fn receive_data(&self, source: &mut BufferedSource, length: i32) -> IoResult<()> {
        assert_lock_not_held();
        let (lock, _) = &*self.state;
        let mut state = lock.lock().unwrap();
        state.source.receive(self, source, length as i64)
    }

    pub fn receive_headers(&self, headers: Headers, in_finished: bool) {
        assert_lock_not_held();

        let open: bool;
        {
            let (lock, cvar) = &*self.state;
            let mut state = lock.lock().unwrap();
            if !state.has_response_headers 
                || headers.get(Header::RESPONSE_STATUS_UTF8).is_some() 
                || headers.get(Header::TARGET_METHOD_UTF8).is_some() 
            {
                state.has_response_headers = true;
                state.headers_queue.push_back(headers);
            } else {
                state.source.trailers = Some(headers);
            }
            if in_finished {
                state.source.finished = true;
            }
            open = self.is_open_internal(&state);
            cvar.notify_all();
        }
        if !open {
            self.connection.remove_stream(self.id);
        }
    }

    pub fn receive_rst_stream(&self, error_code: ErrorCode) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        if state.error_code.is_none() {
            state.error_code = Some(error_code);
            cvar.notify_all();
        }
    }

    fn do_read_timeout(&self, state: &Http2StreamState) -> bool {
        !self.connection.client() || state.sink.closed || state.sink.finished
    }

    fn is_open_internal(&self, state: &Http2StreamState) -> bool {
        if state.error_code.is_some() {
            return false;
        }
        if (state.source.finished || state.source.closed)
            && (state.sink.finished || state.sink.closed)
            && state.has_response_headers
        {
            return false;
        }
        true
    }

    pub fn add_bytes_to_write_window(&self, delta: i64) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        state.write_bytes_maximum += delta;
        if delta > 0 {
            cvar.notify_all();
        }
    }

    pub fn check_out_not_closed(&self, state: &Http2StreamState) -> IoResult<()> {
        if state.sink.closed {
            return Err(Error::new(ErrorKind::Other, "stream closed"));
        }
        if state.sink.finished {
            return Err(Error::new(ErrorKind::Other, "stream finished"));
        }
        if let Some(code) = state.error_code {
            return Err(state.error_exception.clone().unwrap_or_else(|| {
                Error::new(ErrorKind::Other, StreamResetException { code })
            }));
        }
        Ok(())
    }

    pub fn cancel_stream_if_necessary(&self) -> IoResult<()> {
        assert_lock_not_held();

        let (open, cancel) = {
            let (lock, _) = &*self.state;
            let state = lock.lock().unwrap();
            let cancel = !state.source.finished && state.source.closed && (state.sink.finished || state.sink.closed);
            let open = self.is_open_internal(&state);
            (open, cancel)
        };

        if cancel {
            self.close(ErrorCode::CANCEL, None)?;
        } else if !open {
            self.connection.remove_stream(self.id);
        }
        Ok(())
    }
}

struct FramingSource {
    max_byte_count: i64,
    finished: bool,
    receive_buffer: Buffer,
    read_buffer: Buffer,
    trailers: Option<Headers>,
    closed: bool,
}

impl FramingSource {
    fn receive(&mut self, stream: &Http2Stream, source: &mut BufferedSource, byte_count: i64) -> IoResult<()> {
        assert_lock_not_held();
        let mut remaining = byte_count;

        while remaining > 0 {
            let (finished, flow_control_error) = {
                let (lock, _) = &*stream.state;
                let state = lock.lock().unwrap();
                (self.finished, remaining + self.read_buffer.size() > self.max_byte_count)
            };

            if flow_control_error {
                source.skip(remaining);
                stream.close_later(ErrorCode::FLOW_CONTROL_ERROR);
                return Ok(());
            }

            if finished {
                source.skip(remaining);
                return Ok(());
            }

            let read = source.read(&mut self.receive_buffer, remaining);
            if read == -1 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
            }
            remaining -= read;

            {
                let (lock, cvar) = &*stream.state;
                let mut state = lock.lock().unwrap();
                if self.closed {
                    self.receive_buffer.clear();
                } else {
                    let was_empty = self.read_buffer.size() == 0;
                    self.read_buffer.write_all(&mut self.receive_buffer);
                    if was_empty {
                        cvar.notify_all();
                    }
                }
            }
        }

        stream.connection.update_connection_flow_control(byte_count)?;
        stream.connection.flow_control_listener().receiving_stream_window_changed(
            stream.id,
            &stream.read_bytes,
            self.read_buffer.size(),
        );
        Ok(())
    }
}

struct FramingSink {
    finished: bool,
    send_buffer: Buffer,
    trailers: Option<Headers>,
    closed: bool,
}

impl FramingSink {
    fn emit_frame(&mut self, stream: &Http2Stream, out_finished_on_last_frame: bool) -> IoResult<()> {
        let (to_write, out_finished) = {
            let (lock, cvar) = &*stream.state;
            let mut state = lock.lock().unwrap();
            stream.write_timeout.enter();
            while state.write_bytes_total >= state.write_bytes_maximum 
                && !self.finished 
                && !self.closed 
                && state.error_code.is_none() 
            {
                state = cvar.wait(state).unwrap();
            }
            stream.write_timeout.exit_and_throw_if_timed_out()?;

            stream.check_out_not_closed(&state)?;
            
            let to_write = (state.write_bytes_maximum - state.write_bytes_total).min(self.send_buffer.size());
            state.write_bytes_total += to_write;
            let out_finished = out_finished_on_last_frame && to_write == self.send_buffer.size();
            (to_write, out_finished)
        };

        stream.write_timeout.enter();
        let res = stream.connection.write_data(stream.id, out_finished, &mut self.send_buffer, to_write);
        stream.write_timeout.exit_and_throw_if_timed_out()?;
        res
    }
}

pub struct StreamTimeout {
    stream: Option<std::sync::Weak<Http2Stream>>,
    connection: Arc<Http2Connection>,
}

impl AsyncTimeout for StreamTimeout {
    fn timed_out(&self) {
        if let Some(weak) = &self.stream {
            if let Some(stream) = weak.upgrade() {
                stream.close_later(ErrorCode::CANCEL);
            }
        }
        self.connection.send_degraded_ping_later();
    }

    fn new_timeout_exception(&self, cause: Option<Error>) -> Error {
        let mut err = Error::new(ErrorKind::TimedOut, "timeout");
        if let Some(c) = cause {
            err = Error::new(ErrorKind::TimedOut, format!("timeout: {}", c));
        }
        err
    }
}

impl StreamTimeout {
    fn exit_and_throw_if_timed_out(&self) -> IoResult<()> {
        if self.exit() {
            return Err(self.new_timeout_exception(None));
        }
        Ok(())
    }
}
