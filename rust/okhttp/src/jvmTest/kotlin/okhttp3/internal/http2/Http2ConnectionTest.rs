use std::collections::{HashSet, HashMap};
use std::io::{self, Read, Write, EOFError};
use std::sync::{Arc, Mutex, Condvar};
use std::time::{Duration, SystemTime, Instant};
use std::sync::atomic::{AtomicI32, Ordering};
use std::net::TcpStream;

use okio::{Buffer, ByteString};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::Lockable::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::internal::concurrent::TaskFaker::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::TaskRunner::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ConnectionShutdownException::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::FlowControlListener::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Header::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Hpack::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Connection::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2ExchangeCodec::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Stream::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Writer::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Huffman::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::PushObserver::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Settings::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::StreamResetException::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::flowcontrol::WindowCounter::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::PushObserver::*;
use crate::build_logic::settings_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketHandler::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::jvmMain::kotlin::okhttp3::internal::platform::Jdk8WithJettyBootPlatform::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::internal::http2::MockHttp2Peer::*;

// --- Mocking Infrastructure to support the Test Unit ---


impl MockHttp2Peer {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(false),
            frames: Mutex::new(Vec::new()),
            script: Mutex::new(Vec::new()),
            script_ptr: Mutex::new(0),
        }
    }

    pub fn set_client(&self, client: bool) {
        *self.client.lock().unwrap() = client;
    }

    pub fn send_frame(&self) -> MockFrameBuilder {
        MockFrameBuilder { peer: self }
    }

    pub fn accept_frame(&self) {
        self.script.lock().unwrap().push(MockFrame {
            frame_type: -1, // Marker for "accept"
            ..Default::default()
        });
    }

    pub fn play(&self) {
        // In a real mock, this would start the network loop
    }

    pub fn take_frame(&self) -> MockFrame {
        let mut frames = self.frames.lock().unwrap();
        frames.remove(0)
    }

    pub fn frame_count(&self) -> i32 {
        self.frames.lock().unwrap().len() as i32
    }

    pub fn open_socket(&self) -> TcpStream {
        // Mocking a local connection
        TcpStream::connect("127.0.0.1:0").unwrap_or_else(|_| {
            // In test environments, we'd use a mock socket
            panic!("Socket connection failed in mock");
        })
    }

    pub fn max_outbound_data_length(&self) -> usize {
        16384
    }

    pub fn truncate_last_frame(&self, _bytes: usize) {
        // Mock truncation logic
    }

    pub fn close(&self) {
        // Cleanup
    }
}

pub struct MockFrameBuilder<'a> {
    peer: &'a MockHttp2Peer,
}

impl<'a> MockFrameBuilder<'a> {
    pub fn settings(&self, s: Settings) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_SETTINGS,
            settings: Some(s),
            ..Default::default()
        });
        self
    }

    pub fn ping(&self, ack: bool, p1: i32, p2: i32) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_PING,
            payload1: p1,
            payload2: p2,
            ack,
            ..Default::default()
        });
        self
    }

    pub fn window_update(&self, id: i32, inc: i64) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_WINDOW_UPDATE,
            stream_id: id,
            window_size_increment: inc,
            ..Default::default()
        });
        self
    }

    pub fn headers(&self, fin: bool, id: i32, h: Vec<Header>) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_HEADERS,
            out_finished: fin,
            stream_id: id,
            header_block: h,
            ..Default::default()
        });
        self
    }

    pub fn data(&self, fin: bool, id: i32, d: Buffer, len: i32) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_DATA,
            in_finished: fin,
            stream_id: id,
            data: d.read_all().to_vec(),
            payload1: len,
            ..Default::default()
        });
        self
    }

    pub fn push_promise(&self, id: i32, promised: i32, h: Vec<Header>) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_PUSH_PROMISE,
            stream_id: id,
            associated_stream_id: promised,
            header_block: h,
            ..Default::default()
        });
        self
    }

    pub fn go_away(&self, id: i32, err: ErrorCode, data: Vec<u8>) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_GOAWAY,
            stream_id: id,
            error_code: err,
            data,
            ..Default::default()
        });
        self
    }

    pub fn rst_stream(&self, id: i32, err: ErrorCode) -> &Self {
        self.peer.script.lock().unwrap().push(MockFrame {
            frame_type: Http2::TYPE_RST_STREAM,
            stream_id: id,
            error_code: err,
            ..Default::default()
        });
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockFrame {
    pub frame_type: i32,
    pub stream_id: i32,
    pub payload1: i32,
    pub payload2: i32,
    pub ack: bool,
    pub data: Vec<u8>,
    pub out_finished: bool,
    pub in_finished: bool,
    pub header_block: Vec<Header>,
    pub error_code: ErrorCode,
    pub associated_stream_id: i32,
    pub window_size_increment: i64,
    pub settings: Option<Settings>,
}

// --- Test Class Implementation ---

pub struct Http2ConnectionTest {
    peer: MockHttp2Peer,
    task_faker: TaskFaker,
}

impl Http2ConnectionTest {
    pub fn new() -> Self {
        Self {
            peer: MockHttp2Peer::new(),
            task_faker: TaskFaker::new(),
        }
    }

    pub fn tear_down(&mut self) {
        self.peer.close();
    }

    fn connect(&self, peer: &MockHttp2Peer, push_observer: Option<Box<dyn PushObserver>>, listener: Option<Box<dyn Http2ConnectionListener>>) -> Http2Connection {
        let connection = Http2Connection::builder(true, TaskRunner::get_instance())
            .socket(peer.open_socket(), "peer")
            .push_observer(push_observer.unwrap_or_else(|| Box::new(IgnorePushObserver)))
            .listener(listener.unwrap_or_else(|| Box::new(RefuseIncomingStreamsListener)))
            .build();
        
        connection.start(false);

        let ack_frame = peer.take_frame();
        assert_eq!(ack_frame.frame_type, Http2::TYPE_SETTINGS);
        assert_eq!(ack_frame.stream_id, 0);
        assert!(ack_frame.ack);
        
        connection
    }

    fn connect_with_settings(&self, client: bool, settings: Settings) -> Http2Connection {
        self.peer.set_client(client);
        self.peer.send_frame().settings(settings);
        self.peer.accept_frame();
        self.peer.play();
        self.connect(&self.peer, None, None)
    }

    fn await_watchdog_idle(&self) {
        let (tx, rx) = std::sync::mpsc::channel();
        // Mocking AsyncTimeout behavior
        tx.send(()).unwrap();
        rx.recv().unwrap();
    }

    fn assert_stream_data(&self, expected: &str, source: &mut dyn Read) {
        let mut buffer = String::new();
        source.read_to_string(&mut buffer).unwrap();
        assert_eq!(buffer, expected);
    }

    pub fn round_up(num: i32, divisor: i32) -> i32 {
        (num + divisor - 1) / divisor
    }

    // --- Test Methods ---

    pub fn server_pings_client_http2(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.send_frame().ping(false, 2, 3);
        self.peer.accept_frame();
        self.peer.play();

        self.connect(&self.peer, None, None);

        let ping = self.peer.take_frame();
        assert_eq!(ping.frame_type, Http2::TYPE_PING);
        assert_eq!(ping.stream_id, 0);
        assert_eq!(ping.payload1, 2);
        assert_eq!(ping.payload2, 3);
        assert!(ping.ack);
    }

    pub fn peer_http2_server_lowers_initial_window_size(&self) {
        let mut initial = Settings::default();
        initial.set(Settings::INITIAL_WINDOW_SIZE, 1684);
        let mut shouldnt_impact = Settings::default();
        shouldnt_impact.set(Settings::INITIAL_WINDOW_SIZE, 3368);
        
        self.peer.send_frame().settings(initial);
        self.peer.accept_frame();
        self.peer.send_frame().settings(shouldnt_impact);
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.play();
        
        let connection = self.connect(&self.peer, None, None);
        let ack_frame = self.peer.take_frame();
        assert_eq!(ack_frame.frame_type, Http2::TYPE_SETTINGS);
        assert_eq!(ack_frame.stream_id, 0);
        assert!(ack_frame.ack);

        let stream = connection.new_stream(vec![Header::new("a", "android")], false);
        assert_eq!(connection.peer_settings().initial_window_size, 3368);
        assert_eq!(stream.write_bytes_total(), 0);
        assert_eq!(stream.write_bytes_maximum(), 3368);
    }

    pub fn peer_http2_server_zeros_compression_table(&self) {
        let mut settings = Settings::default();
        settings.set(Settings::HEADER_TABLE_SIZE, 0);
        let connection = self.connect_with_settings(false, settings);

        assert_eq!(connection.peer_settings().header_table_size, 0);
        let writer = connection.writer();
        assert_eq!(writer.hpack_writer().dynamic_table_byte_count(), 0);
        assert_eq!(writer.hpack_writer().header_table_size_setting(), 0);
    }

    pub fn peer_http2_client_disables_push(&self) {
        let mut settings = Settings::default();
        settings.set(Settings::ENABLE_PUSH, 0);
        let connection = self.connect_with_settings(false, settings);

        assert!(!connection.peer_settings().get_enable_push(true));
    }

    pub fn peer_increases_max_frame_size(&self) {
        let new_max = 0x4001;
        let mut settings = Settings::default();
        settings.set(Settings::MAX_FRAME_SIZE, new_max);
        let connection = self.connect_with_settings(true, settings);

        assert_eq!(connection.peer_settings().get_max_frame_size(-1), new_max);
        assert_eq!(connection.writer().max_data_length(), new_max);
    }

    pub fn peer_sets_zero_flow_control(&self) {
        self.peer.set_client(true);
        let mut s = Settings::default();
        s.set(Settings::INITIAL_WINDOW_SIZE, 0);
        self.peer.send_frame().settings(s);
        self.peer.accept_frame();
        self.peer.send_frame().window_update(0, 10);
        self.peer.accept_frame();
        self.peer.send_frame().ping(true, Http2Connection::AWAIT_PING, 0);
        self.peer.accept_frame();
        self.peer.send_frame().window_update(3, 5);
        self.peer.accept_frame();
        self.peer.send_frame().window_update(3, 5);
        self.peer.accept_frame();
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        connection.write_ping_and_await_pong();
        let stream = connection.new_stream(vec![Header::new("a", "android")], true);
        let mut sink = stream.sink().buffer();
        sink.write_all(b"abcdefghi").unwrap();
        sink.flush().unwrap();

        self.peer.take_frame(); // PING
        let headers = self.peer.take_frame();
        assert_eq!(headers.frame_type, Http2::TYPE_HEADERS);
        let data1 = self.peer.take_frame();
        assert_eq!(data1.frame_type, Http2::TYPE_DATA);
        assert_eq!(data1.stream_id, 3);
        assert_eq!(data1.data, b"abcde");
        let data2 = self.peer.take_frame();
        assert_eq!(data2.frame_type, Http2::TYPE_DATA);
        assert_eq!(data2.stream_id, 3);
        assert_eq!(data2.data, b"fghi");
    }

    pub fn discarded_data_frames_are_counted(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(false, 3, vec![Header::new("a", "apple")]);
        self.peer.send_frame().data(false, 3, Buffer::new().write(vec![0u8; 1024]), 1024);
        self.peer.accept_frame();
        self.peer.send_frame().data(true, 3, Buffer::new().write(vec![0u8; 1024]), 1024);
        self.peer.accept_frame();
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        let stream1 = connection.new_stream(vec![Header::new("b", "bark")], false);
        let mut source = stream1.source();
        let mut buffer = Buffer::new();
        while buffer.size() != 1024 {
            source.read(&mut buffer, 1024).unwrap();
        }
        stream1.close(ErrorCode::CANCEL, None);
        
        let frame1 = self.peer.take_frame();
        assert_eq!(frame1.frame_type, Http2::TYPE_HEADERS);
        let frame2 = self.peer.take_frame();
        assert_eq!(frame2.frame_type, Http2::TYPE_RST_STREAM);
        let frame3 = self.peer.take_frame();
        assert_eq!(frame3.frame_type, Http2::TYPE_RST_STREAM);
        assert_eq!(connection.read_bytes().acknowledged, 0);
        assert_eq!(connection.read_bytes().total, 2048);
    }

    pub fn receive_go_away_http2(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().go_away(3, ErrorCode::PROTOCOL_ERROR, vec![]);
        self.peer.accept_frame();
        self.peer.send_frame().ping(true, Http2Connection::AWAIT_PING, 0);
        self.peer.accept_frame();
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        let stream1 = connection.new_stream(vec![Header::new("a", "android")], true);
        let stream2 = connection.new_stream(vec![Header::new("b", "banana")], true);
        connection.write_ping_and_await_pong();
        
        let mut sink1 = stream1.sink().buffer();
        let mut sink2 = stream2.sink().buffer();
        sink1.write_all(b"abc").unwrap();
        
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            sink2.write_all(b"abc").unwrap();
            sink2.flush().unwrap();
        }));
        assert!(res.is_err());

        sink1.write_all(b"def").unwrap();
        sink1.close().unwrap();
        
        let res_new = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            connection.new_stream(vec![Header::new("c", "cola")], true);
        }));
        assert!(res_new.is_err());

        assert!(stream1.is_open());
        assert!(!stream2.is_open());
        assert_eq!(connection.open_stream_count(), 1);

        let syn1 = self.peer.take_frame();
        assert_eq!(syn1.frame_type, Http2::TYPE_HEADERS);
        let syn2 = self.peer.take_frame();
        assert_eq!(syn2.frame_type, Http2::TYPE_HEADERS);
        let ping = self.peer.take_frame();
        assert_eq!(ping.frame_type, Http2::TYPE_PING);
        let data1 = self.peer.take_frame();
        assert_eq!(data1.frame_type, Http2::TYPE_DATA);
        assert_eq!(data1.stream_id, 3);
        assert_eq!(data1.data, b"abcdef");
    }

    pub fn read_sends_window_update_http2(&self) {
        let window_size = 100;
        let threshold = 50;

        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(false, 3, vec![Header::new("a", "android")]);
        for _ in 0..3 {
            self.peer.send_frame().data(false, 3, Buffer::new().write(vec![0u8; 24]), 24);
            self.peer.send_frame().data(false, 3, Buffer::new().write(vec![0u8; 25]), 25);
            self.peer.send_frame().data(false, 3, Buffer::new().write(vec![0u8; 1]), 1);
            self.peer.accept_frame();
            self.peer.accept_frame();
        }
        self.peer.send_frame().data(true, 3, Buffer::new(), 0);
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        connection.ok_http_settings().set(Settings::INITIAL_WINDOW_SIZE, window_size);
        let stream = connection.new_stream(vec![Header::new("b", "banana")], false);
        
        assert_eq!(stream.read_bytes().acknowledged, 0);
        assert_eq!(stream.read_bytes().total, 0);
        assert_eq!(stream.take_headers(), vec![Header::new("a", "android")]);
        
        let mut source = stream.source();
        let mut buffer = Buffer::new();
        while source.read(&mut buffer, 1).unwrap() != -1 {}
        assert_eq!(buffer.size(), 150);

        let syn = self.peer.take_frame();
        assert_eq!(syn.frame_type, Http2::TYPE_HEADERS);
        for _ in 0..3 {
            let mut ids = Vec::new();
            for _ in 0..2 {
                let wu = self.peer.take_frame();
                assert_eq!(wu.frame_type, Http2::TYPE_WINDOW_UPDATE);
                ids.push(wu.stream_id);
                assert_eq!(wu.window_size_increment, threshold as i64);
            }
            assert!(ids.contains(&0));
            assert!(ids.contains(&3));
        }
    }

    pub fn server_sends_empty_data_client_doesnt_send_window_update_http2(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(false, 3, vec![Header::new("a", "android")]);
        self.peer.send_frame().data(true, 3, Buffer::new(), 0);
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        let client = connection.new_stream(vec![Header::new("b", "banana")], false);
        let mut buf = Buffer::new();
        assert_eq!(client.source().read(&mut buf, 1).unwrap(), -1);

        let syn = self.peer.take_frame();
        assert_eq!(syn.frame_type, Http2::TYPE_HEADERS);
        assert_eq!(self.peer.frame_count(), 5);
    }

    pub fn client_sends_empty_data_server_doesnt_send_window_update_http2(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(false, 3, vec![Header::new("a", "android")]);
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        let client = connection.new_stream(vec![Header::new("b", "banana")], true);
        let mut out = client.sink().buffer();
        out.write_all(&[]).unwrap();
        out.flush().unwrap();
        out.close().unwrap();

        assert_eq!(self.peer.take_frame().frame_type, Http2::TYPE_HEADERS);
        assert_eq!(self.peer.take_frame().frame_type, Http2::TYPE_DATA);
        assert_eq!(self.peer.frame_count(), 5);
    }

    pub fn max_frame_size_honored(&self) {
        let mut buff = vec![b'*'; self.peer.max_outbound_data_length() + 1];
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(false, 3, vec![Header::new("a", "android")]);
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        let stream = connection.new_stream(vec![Header::new("b", "banana")], true);
        let mut out = stream.sink().buffer();
        out.write_all(&buff).unwrap();
        out.flush().unwrap();
        out.close().unwrap();

        let syn = self.peer.take_frame();
        assert_eq!(syn.frame_type, Http2::TYPE_HEADERS);
        let mut data = self.peer.take_frame();
        assert_eq!(data.data.len(), self.peer.max_outbound_data_length());
        data = self.peer.take_frame();
        assert_eq!(data.data.len(), 1);
    }

    pub fn push_promise_stream(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(false, 3, vec![Header::new("a", "android")]);
        
        let req_headers = vec![
            Header::new(Header::TARGET_METHOD, "GET"),
            Header::new(Header::TARGET_SCHEME, "https"),
            Header::new(Header::TARGET_AUTHORITY, "squareup.com"),
            Header::new(Header::TARGET_PATH, "/cached"),
        ];
        self.peer.send_frame().push_promise(3, 2, req_headers.clone());
        
        let res_headers = vec![Header::new(Header::RESPONSE_STATUS, "200")];
        self.peer.send_frame().headers(true, 2, res_headers.clone());
        self.peer.send_frame().data(true, 3, Buffer::new(), 0);
        self.peer.play();

        let observer = Arc::new(RecordingPushObserver::new());
        let connection = self.connect(&self.peer, Some(Box::new(observer.clone())), Some(Box::new(RefuseIncomingStreamsListener)));
        let client = connection.new_stream(vec![Header::new("b", "banana")], false);
        let mut buf = Buffer::new();
        assert_eq!(client.source().read(&mut buf, 1).unwrap(), -1);

        assert_eq!(self.peer.take_frame().frame_type, Http2::TYPE_HEADERS);
        assert_eq!(observer.take_event(), Some(req_headers));
        assert_eq!(observer.take_event(), Some(res_headers));
    }

    pub fn double_push_promise(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.send_frame().push_promise(3, 2, vec![Header::new("a", "android")]);
        self.peer.accept_frame();
        self.peer.send_frame().push_promise(3, 2, vec![Header::new("b", "banana")]);
        self.peer.accept_frame();
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        connection.new_stream(vec![Header::new("b", "banana")], false);

        assert_eq!(self.peer.take_frame().frame_type, Http2::TYPE_HEADERS);
        assert_eq!(self.peer.take_frame().error_code, ErrorCode::PROTOCOL_ERROR);
    }

    pub fn push_promise_streams_automatically_cancel(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.send_frame().push_promise(3, 2, vec![
            Header::new(Header::TARGET_METHOD, "GET"),
            Header::new(Header::TARGET_SCHEME, "https"),
            Header::new(Header::TARGET_AUTHORITY, "squareup.com"),
            Header::new(Header::TARGET_PATH, "/cached"),
        ]);
        self.peer.send_frame().headers(true, 2, vec![Header::new(Header::RESPONSE_STATUS, "200")]);
        self.peer.accept_frame();
        self.peer.play();

        self.connect(&self.peer, Some(Box::new(IgnorePushObserver)), Some(Box::new(RefuseIncomingStreamsListener)));

        let rst = self.peer.take_frame();
        assert_eq!(rst.frame_type, Http2::TYPE_RST_STREAM);
        assert_eq!(rst.stream_id, 2);
        assert_eq!(rst.error_code, ErrorCode::CANCEL);
    }

    pub fn socket_exception_while_writing_headers(&self) {
        self.peer.accept_frame();
        self.peer.play();
        let long_string = "a".repeat(Http2::INITIAL_MAX_FRAME_SIZE + 1);
        let socket = self.peer.open_socket();
        
        let connection = Http2Connection::builder(true, TaskRunner::get_instance())
            .socket(socket, "peer")
            .push_observer(Box::new(IgnorePushObserver))
            .build();
        
        connection.start(false);
        // Simulate socket shutdown
        // In Rust, we'd use shutdown(Write)
        
        let res1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            connection.new_stream(vec![Header::new("a", &long_string)], false);
        }));
        assert!(res1.is_err());
        
        let res2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            connection.new_stream(vec![Header::new("b", &long_string)], false);
        }));
        assert!(res2.is_err());
    }

    pub fn client_creates_stream_and_server_replies(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(false, 3, vec![Header::new("a", "android")]);
        self.peer.send_frame().data(true, 3, Buffer::new().write("robot"), 5);
        self.peer.accept_frame();
        self.peer.send_frame().ping(true, Http2Connection::AWAIT_PING, 0);
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        let stream = connection.new_stream(vec![Header::new("b", "banana")], true);
        let mut out = stream.sink().buffer();
        out.write_all(b"c3po").unwrap();
        out.close().unwrap();
        
        assert_eq!(stream.take_headers(), vec![Header::new("a", "android")]);
        self.assert_stream_data("robot", &mut stream.source());
        connection.write_ping_and_await_pong();
        assert_eq!(connection.open_stream_count(), 0);

        let syn = self.peer.take_frame();
        assert_eq!(syn.frame_type, Http2::TYPE_HEADERS);
        assert!(!syn.out_finished);
        assert_eq!(syn.stream_id, 3);
        assert_eq!(syn.header_block, vec![Header::new("b", "banana")]);
        let req_data = self.peer.take_frame();
        assert_eq!(req_data.data, b"c3po");
    }

    pub fn server_finishes_stream_with_headers(&self) {
        self.peer.send_frame().settings(Settings::default());
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.accept_frame();
        self.peer.send_frame().headers(true, 3, vec![Header::new("headers", "bam")]);
        self.peer.send_frame().ping(true, Http2Connection::AWAIT_PING, 0);
        self.peer.play();

        let connection = self.connect(&self.peer, None, None);
        let stream = connection.new_stream(vec![Header::new("a", "artichaut")], false);
        connection.write_ping_and_await_pong();
        assert_eq!(stream.take_headers(), vec![Header::new("headers", "bam")]);
        assert_eq!(stream.peek_trailers(), vec![]);
        assert_eq!(connection.open_stream_count(), 0);

        let syn = self.peer.take_frame();
        assert_eq!(syn.frame_type, Http2::TYPE_HEADERS);
        assert!(!syn.out_finished);
        assert_eq!(syn.stream_id, 3);
        assert_eq!(syn.header_block, vec![Header::new("a", "artich