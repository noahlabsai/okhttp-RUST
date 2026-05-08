use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use log::info;

// Assuming these types are defined in the crate as per the provided context
use crate::okhttp3::internal::http2::{
    Http2, Http2Reader, Http2Writer, Header, Settings, ErrorCode, 
    BufferedSource, ByteString
};

/// Replays prerecorded outgoing frames and records incoming frames.
pub struct MockHttp2Peer {
    state: Arc<Mutex<PeerState>>,
    executor_shutdown: Arc<Mutex<bool>>,
    in_frames_tx: Sender<InFrame>,
    in_frames_rx: Mutex<Receiver<InFrame>>,
}

struct PeerState {
    frame_count: i32,
    client: bool,
    bytes_out: Vec<u8>, // Using Vec<u8> as a Buffer equivalent
    writer: Http2Writer<Vec<u8>>,
    out_frames: Vec<OutFrame>,
    port: u16,
    server_socket: Option<TcpListener>,
    socket: Option<TcpStream>,
}

#[derive(Debug, Clone, PartialEq)]
struct OutFrame {
    sequence: i32,
    start: i64,
    truncated: bool,
}

pub struct InFrame {
    pub sequence: i32,
    pub reader: Arc<Mutex<Http2Reader<Box<dyn Read + Send>>>>,
    pub frame_type: i32,
    pub clear_previous: bool,
    pub out_finished: bool,
    pub in_finished: bool,
    pub stream_id: i32,
    pub associated_stream_id: i32,
    pub error_code: Option<ErrorCode>,
    pub window_size_increment: i64,
    pub header_block: Option<Vec<Header>>,
    pub data: Option<Vec<u8>>,
    pub settings: Option<Settings>,
    pub ack: bool,
    pub payload1: i32,
    pub payload2: i32,
}

impl Http2Reader::Handler for InFrame {
    fn settings(&mut self, clear_previous: bool, settings: Settings) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_SETTINGS;
        self.clear_previous = clear_previous;
        self.settings = Some(settings);
    }

    fn ack_settings(&mut self) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_SETTINGS;
        self.ack = true;
    }

    fn headers(&mut self, in_finished: bool, stream_id: i32, associated_stream_id: i32, header_block: Vec<Header>) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_HEADERS;
        self.in_finished = in_finished;
        self.stream_id = stream_id;
        self.associated_stream_id = associated_stream_id;
        self.header_block = Some(header_block);
    }

    fn data(&mut self, in_finished: bool, stream_id: i32, source: &mut dyn BufferedSource, length: i32) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_DATA;
        self.in_finished = in_finished;
        self.stream_id = stream_id;
        let mut buf = vec![0u8; length as usize];
        source.read_exact(&mut buf).expect("Failed to read data frame");
        self.data = Some(buf);
    }

    fn rst_stream(&mut self, stream_id: i32, error_code: ErrorCode) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_RST_STREAM;
        self.stream_id = stream_id;
        self.error_code = Some(error_code);
    }

    fn ping(&mut self, ack: bool, payload1: i32, payload2: i32) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_PING;
        self.ack = ack;
        self.payload1 = payload1;
        self.payload2 = payload2;
    }

    fn go_away(&mut self, last_good_stream_id: i32, error_code: ErrorCode, debug_data: ByteString) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_GOAWAY;
        self.stream_id = last_good_stream_id;
        self.error_code = Some(error_code);
        self.data = Some(debug_data.to_bytes());
    }

    fn window_update(&mut self, stream_id: i32, window_size_increment: i64) {
        assert_eq!(self.frame_type, -1);
        self.frame_type = Http2::TYPE_WINDOW_UPDATE;
        self.stream_id = stream_id;
        self.window_size_increment = window_size_increment;
    }

    fn priority(&mut self, _stream_id: i32, _stream_dependency: i32, _weight: i32, _exclusive: bool) {
        panic!("UnsupportedOperationException");
    }

    fn push_promise(&mut self, stream_id: i32, associated_stream_id: i32, header_block: Vec<Header>) {
        self.frame_type = Http2::TYPE_PUSH_PROMISE;
        self.stream_id = stream_id;
        self.associated_stream_id = associated_stream_id;
        self.header_block = Some(header_block);
    }

    fn alternate_service(&mut self, _stream_id: i32, _origin: String, _protocol: ByteString, _host: String, _port: i32, _max_age: i64) {
        panic!("UnsupportedOperationException");
    }
}

impl MockHttp2Peer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let bytes_out = Vec::new();
        let writer = Http2Writer::new(bytes_out.clone(), false);
        
        MockHttp2Peer {
            state: Arc::new(Mutex::new(PeerState {
                frame_count: 0,
                client: false,
                bytes_out: Vec::new(),
                writer,
                out_frames: Vec::new(),
                port: 0,
                server_socket: None,
                socket: None,
            })),
            executor_shutdown: Arc::new(Mutex::new(false)),
            in_frames_tx: tx,
            in_frames_rx: Mutex::new(rx),
        }
    }

    pub fn set_client(&self, client: bool) {
        let mut state = self.state.lock().unwrap();
        if state.client == client {
            return;
        }
        state.client = client;
        state.writer = Http2Writer::new(state.bytes_out.clone(), client);
    }

    pub fn accept_frame(&self) {
        let mut state = self.state.lock().unwrap();
        state.frame_count += 1;
    }

    pub fn max_outbound_data_length(&self) -> i32 {
        self.state.lock().unwrap().writer.max_data_length()
    }

    pub fn frame_count(&self) -> i32 {
        self.state.lock().unwrap().frame_count
    }

    pub fn send_frame(&self) -> i32 {
        let mut state = self.state.lock().unwrap();
        let seq = state.frame_count;
        state.frame_count += 1;
        state.out_frames.push(OutFrame {
            sequence: seq,
            start: state.bytes_out.len() as i64,
            truncated: false,
        });
        seq
    }

    pub fn truncate_last_frame(&self, length: i32) {
        let mut state = self.state.lock().unwrap();
        let last_frame = state.out_frames.pop().expect("No frames to truncate");
        
        let current_size = state.bytes_out.len() as i64;
        assert!(length as i64 < current_size - last_frame.start);

        let full_buffer = state.bytes_out.clone();
        state.bytes_out.clear();
        
        let end_pos = (last_frame.start + length as i64) as usize;
        state.bytes_out.extend_from_slice(&full_buffer[..end_pos]);
        
        state.out_frames.push(OutFrame {
            sequence: last_frame.sequence,
            start: last_frame.start,
            truncated: true,
        });
    }

    pub fn take_frame(&self) -> InFrame {
        self.in_frames_rx.lock().unwrap().recv().expect("Channel closed")
    }

    pub fn play(&self) {
        let mut state = self.state.lock().unwrap();
        assert!(state.server_socket.is_none());
        
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
        let port = listener.local_addr().unwrap().port();
        state.port = port;
        state.server_socket = Some(listener);

        let state_clone = Arc::clone(&self.state);
        let shutdown_clone = Arc::clone(&self.executor_shutdown);
        let tx_clone = self.in_frames_tx.clone();

        thread::spawn(move || {
            let result = (|| -> io::Result<()> {
                let mut state_lock = state_clone.lock().unwrap();
                let listener = state_lock.server_socket.as_ref().unwrap();
                let (socket, _) = listener.accept()?;
                state_lock.socket = Some(socket.try_clone()?);
                
                // Check shutdown
                if *shutdown_clone.lock().unwrap() {
                    return Ok(());
                }
                
                drop(state_lock); // Release lock for read_and_write_frames
                
                let mut state_lock = state_clone.lock().unwrap();
                let socket = state_lock.socket.as_ref().unwrap();
                let mut output_stream = socket.try_clone()?;
                let mut input_stream = socket.try_clone()?;
                
                let client = state_lock.client;
                let frame_count = state_lock.frame_count;
                let out_frames = state_lock.out_frames.clone();
                let out_bytes = state_lock.bytes_out.clone();
                
                let reader = Http2Reader::new(Box::new(input_stream), client);
                let reader_arc = Arc::new(Mutex::new(reader));
                
                let mut out_frames_iter = out_frames.into_iter();
                let mut next_out_frame: Option<OutFrame> = None;

                for i in 0..frame_count {
                    if next_out_frame.is_none() {
                        next_out_frame = out_frames_iter.next();
                    }

                    if let Some(ref frame) = next_out_frame {
                        if frame.sequence == i {
                            let start = frame.start as usize;
                            let (truncated, end) = if let Some(next) = out_frames_iter.next() {
                                next_out_frame = Some(next);
                                (false, next.start as usize)
                            } else {
                                (frame.truncated, out_bytes.len())
                            };

                            let length = end - start;
                            output_stream.write_all(&out_bytes[start..end])?;

                            if truncated {
                                return socket.shutdown(std::net::Shutdown::Both);
                            }
                            continue;
                        }
                    }
                    
                    let mut in_frame = InFrame {
                        sequence: i,
                        reader: Arc::clone(&reader_arc),
                        frame_type: -1,
                        clear_previous: false,
                        out_finished: false,
                        in_finished: false,
                        stream_id: 0,
                        associated_stream_id: 0,
                        error_code: None,
                        window_size_increment: 0,
                        header_block: None,
                        data: None,
                        settings: None,
                        ack: false,
                        payload1: 0,
                        payload2: 0,
                    };
                    
                    reader_arc.lock().unwrap().next_frame(false, &mut in_frame);
                    tx_clone.send(in_frame).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                }
                Ok(())
            })();

            if let Err(e) = result {
                info!("MockHttp2Peer done: {}", e);
            }
        });
    }

    pub fn open_socket(&self) -> TcpStream {
        let port = self.state.lock().unwrap().port;
        TcpStream::connect(format!("127.0.0.1:{}", port)).expect("Failed to connect")
    }

    pub fn close(&self) {
        let mut shutdown = self.executor_shutdown.lock().unwrap();
        *shutdown = true;
        
        let mut state = self.state.lock().unwrap();
        if let Some(socket) = state.socket.take() {
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
        if let Some(_listener) = state.server_socket.take() {
            // TcpListener is closed when dropped
        }
    }
}

impl std::fmt::Display for MockHttp2Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let port = self.state.lock().unwrap().port;
        write!(f, "MockHttp2Peer[{}]", port)
    }
}

impl Drop for MockHttp2Peer {
    fn drop(&mut self) {
        self.close();
    }
}
