use std::collections::HashSet;
use std::io::{self, Read, Write, Error as IoError, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, atomic::{AtomicI32, Ordering}};
use std::time::{Duration, Instant};
use std::sync::mpsc::{channel, Receiver, Sender};

use okio::{Buffer, BufferedSink, BufferedSource, ByteString};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{HttpUrl, Headers, Protocol, Request, Response};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::{TaskRunner, TaskQueue};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::HttpMethod;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::{ErrorCode, Header, Http2Connection, Http2Stream};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketEffect::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::{
    MockWebServerSocket, RecordedRequest, RequestLine, ThrottledSink, TriggerSink, decode_request_line,
    DEFAULT_REQUEST_LINE_HTTP_1, DEFAULT_REQUEST_LINE_HTTP_2
};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::{RealWebSocket, WebSocketExtensions, WebSocketProtocol};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::ThrottledSink::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::TriggerSink::*;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::Dispatcher::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Stream::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::HttpMethod::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

const CLIENT_AUTH_NONE: i32 = 0;
const CLIENT_AUTH_REQUESTED: i32 = 1;
const CLIENT_AUTH_REQUIRED: i32 = 2;

pub trait Dispatcher: Send + Sync {
    fn dispatch(&self, request: RecordedRequest) -> MockResponse;
    fn peek(&self) -> MockResponse;
    fn close(&self);
}

pub struct QueueDispatcher {
    queue: Mutex<Vec<MockResponse>>,
}

impl QueueDispatcher {
    pub fn new() -> Self {
        Self { queue: Mutex::new(Vec::new()) }
    }
    pub fn enqueue(&self, response: MockResponse) {
        self.queue.lock().unwrap().push(response);
    }
}

impl Dispatcher for QueueDispatcher {
    fn dispatch(&self, _request: RecordedRequest) -> MockResponse {
        let mut q = self.queue.lock().unwrap();
        if q.is_empty() {
            return MockResponse::default();
        }
        q.remove(0)
    }
    fn peek(&self) -> MockResponse {
        let q = self.queue.lock().unwrap();
        q.first().cloned().unwrap_or_else(MockResponse::default)
    }
    fn close(&self) {}
}

pub struct MockWebServer {
    task_runner: Arc<TaskRunner>,
    request_queue: Arc<Mutex<Vec<RecordedRequest>>>,
    open_client_sockets: Arc<Mutex<HashSet<SocketAddr>>>,
    open_connections: Arc<Mutex<HashSet<usize>>>,
    atomic_request_count: AtomicI32,
    server_socket_factory: Mutex<Option<Box<dyn ServerSocketFactory + Send + Sync>>>,
    server_socket: Mutex<Option<TcpListener>>,
    socket_address: Mutex<Option<SocketAddr>>,
    ssl_socket_factory: Mutex<Option<Box<dyn SslSocketFactory + Send + Sync>>>,
    client_auth: Mutex<i32>,
    closed: Mutex<bool>,
    pub body_limit: Mutex<i64>,
    pub dispatcher: Mutex<Box<dyn Dispatcher>>,
    pub protocol_negotiation_enabled: Mutex<bool>,
    pub protocols: Mutex<Vec<Protocol>>,
}

trait ServerSocketFactory: Send + Sync {
    fn create_server_socket(&self) -> io::Result<TcpListener>;
}

trait SslSocketFactory: Send + Sync {
    fn create_socket(&self, stream: TcpStream, host: String, port: u16, auto_close: bool) -> io::Result<Box<dyn SslSocket>>;
}

trait SslSocket: Read + Write + Send + Sync {
    fn start_handshake(&self) -> io::Result<()>;
    fn set_use_client_mode(&self, mode: bool) -> io::Result<()>;
    fn set_need_client_auth(&self, need: bool) -> io::Result<()>;
    fn set_want_client_auth(&self, want: bool) -> io::Result<()>;
    fn get_selected_protocol(&self) -> Option<String>;
    fn get_remote_addr(&self) -> SocketAddr;
}

impl MockWebServer {
    pub fn new() -> Arc<Self> {
        let backend = Box::new(crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::RealBackend::new());
        let logger = Arc::new(crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::DefaultLogger);
        let task_runner = TaskRunner::new(backend, logger);

        Arc::new(Self {
            task_runner,
            request_queue: Arc::new(Mutex::new(Vec::new())),
            open_client_sockets: Arc::new(Mutex::new(HashSet::new())),
            open_connections: Arc::new(Mutex::new(HashSet::new())),
            atomic_request_count: AtomicI32::new(0),
            server_socket_factory: Mutex::new(None),
            server_socket: Mutex::new(None),
            socket_address: Mutex::new(None),
            ssl_socket_factory: Mutex::new(None),
            client_auth: Mutex::new(CLIENT_AUTH_NONE),
            closed: Mutex::new(false),
            body_limit: Mutex::new(i64::MAX),
            dispatcher: Mutex::new(Box::new(QueueDispatcher::new())),
            protocol_negotiation_enabled: Mutex::new(true),
            protocols: Mutex::new(vec![Protocol::Http2, Protocol::Http11]),
        })
    }

    pub fn request_count(&self) -> i32 {
        self.atomic_request_count.load(Ordering::SeqCst)
    }

    pub fn url(&self, path: &str) -> HttpUrl {
        let is_https = self.ssl_socket_factory.lock().unwrap().is_some();
        let addr = self.socket_address.lock().unwrap().expect("call start() first");
        HttpUrl::builder()
            .scheme(if is_https { "https" } else { "http" })
            .host(addr.ip().to_string().as_str())
            .port(addr.port())
            .build()
            .unwrap()
            .resolve(path)
            .expect("invalid path")
    }

    pub fn use_https(&self, factory: Box<dyn SslSocketFactory + Send + Sync>) {
        *self.ssl_socket_factory.lock().unwrap() = Some(factory);
    }

    pub fn no_client_auth(&self) {
        *self.client_auth.lock().unwrap() = CLIENT_AUTH_NONE;
    }

    pub fn request_client_auth(&self) {
        *self.client_auth.lock().unwrap() = CLIENT_AUTH_REQUESTED;
    }

    pub fn require_client_auth(&self) {
        *self.client_auth.lock().unwrap() = CLIENT_AUTH_REQUIRED;
    }

    pub fn take_request(&self) -> RecordedRequest {
        loop {
            let mut q = self.request_queue.lock().unwrap();
            if !q.is_empty() {
                return q.remove(0);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn take_request_timeout(&self, timeout: Duration) -> Option<RecordedRequest> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let mut q = self.request_queue.lock().unwrap();
            if !q.is_empty() {
                return Some(q.remove(0));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }

    pub fn enqueue(&self, response: MockResponse) {
        let mut disp = self.dispatcher.lock().unwrap();
        if let Some(qd) = disp.as_any().downcast_ref::<QueueDispatcher>() {
            qd.enqueue(response);
        } else {
            panic!("Default dispatcher replaced");
        }
    }

    pub fn start(self: &Arc<Self>, port: u16) -> io::Result<()> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        self.start_with_address(addr)
    }

    fn start_with_address(self: &Arc<Self>, addr: SocketAddr) -> io::Result<()> {
        let mut closed = self.closed.lock().unwrap();
        if *closed { return Err(IoError::new(ErrorKind::Other, "close() already called")); }

        let mut sa_lock = self.socket_address.lock().unwrap();
        if let Some(already) = *sa_lock {
            if already.ip() != addr.ip() { return Err(IoError::new(ErrorKind::Other, "unexpected address")); }
            if addr.port() != 0 && addr.port() != already.port() { return Err(IoError::new(ErrorKind::Other, "unexpected port")); }
            return Ok(());
        }

        let listener = if let Some(factory) = &*self.server_socket_factory.lock().unwrap() {
            factory.create_server_socket()?
        } else {
            TcpListener::bind(addr)?
        };

        let bound_addr = listener.local_addr()?;
        *sa_lock = Some(bound_addr);
        *self.server_socket.lock().unwrap() = Some(listener);

        let server_clone = Arc::clone(self);
        let listener_clone = self.server_socket.lock().unwrap().as_ref().unwrap().try_clone()?;
        
        server_clone.task_runner.new_queue().execute("MockWebServer Acceptor", false, move || {
            server_clone.accept_connections(listener_clone);
        });

        Ok(())
    }

    fn accept_connections(self: &Arc<Self>, listener: TcpListener) {
        let mut next_connection_index = 0;
        loop {
            match listener.accept() {
                Ok((socket, addr)) => {
                    let peek = self.dispatcher.lock().unwrap().peek();
                    if let SocketEffect::CloseSocket { .. } = peek.on_request_start {
                        self.dispatch_bookkeeping_request(next_connection_index, 0, MockWebServerSocket::new(socket));
                        next_connection_index += 1;
                        let _ = socket.shutdown(std::net::Shutdown::Both);
                    } else {
                        self.open_client_sockets.lock().unwrap().insert(addr);
                        self.serve_connection(next_connection_index, socket, peek);
                        next_connection_index += 1;
                    }
                }
                Err(e) => {
                    if *self.closed.lock().unwrap() { return; }
                    eprintln!("Accept error: {}", e);
                }
            }
        }
    }

    fn serve_connection(self: &Arc<Self>, index: i32, socket: TcpStream, peek: MockResponse) {
        let server_clone = Arc::clone(self);
        let addr = socket.peer_addr().unwrap_or(SocketAddr::from([127,0,0,1], 0));
        self.task_runner.new_queue().execute(&format!("MockWebServer {}", addr), false, move || {
            let handler = SocketHandler {
                server: server_clone,
                connection_index: index,
                raw: socket,
                first_exchange_peek: peek,
            };
            if let Err(e) = handler.handle() {
                eprintln!("Connection handler error: {}", e);
            }
        });
    }

    pub fn close(&self) {
        let mut closed = self.closed.lock().unwrap();
        if *closed { return; }
        *closed = true;

        if let Some(listener) = self.server_socket.lock().unwrap().take() {
            let _ = listener.shutdown(std::net::Shutdown::Both); // Not exactly possible on listener, but close it
        }

        let mut sockets = self.open_client_sockets.lock().unwrap();
        for addr in sockets.drain() {
            // In a real impl, we'd track the actual TcpStream objects
        }
        
        let mut connections = self.open_connections.lock().unwrap();
        connections.clear();
        
        self.dispatcher.lock().unwrap().close();
    }

    fn dispatch_bookkeeping_request(self: &Arc<Self>, conn_idx: i32, exch_idx: i32, socket: MockWebServerSocket) {
        let request = RecordedRequest {
            request_line: DEFAULT_REQUEST_LINE_HTTP_1,
            headers: Headers::empty().clone(),
            chunk_sizes: None,
            body_size: 0,
            body: None,
            connection_index: conn_idx,
            exchange_index: exch_idx,
            socket,
            failure: None,
        };
        self.atomic_request_count.fetch_add(1, Ordering::SeqCst);
        self.request_queue.lock().unwrap().push(request.clone());
        self.dispatcher.lock().unwrap().dispatch(request);
    }
}

struct SocketHandler {
    server: Arc<MockWebServer>,
    connection_index: i32,
    raw: TcpStream,
    first_exchange_peek: MockResponse,
}

impl SocketHandler {
    fn handle(&self) -> io::Result<()> {
        let mut next_exchange_index = 0;
        
        if !self.process_tunnel_requests()? { return Ok(()); }

        let protocol: Protocol;
        let socket: MockWebServerSocket;

        let ssl_factory = self.server.ssl_socket_factory.lock().unwrap();
        if let Some(factory) = ssl_factory.as_ref() {
            if self.first_exchange_peek.fail_handshake {
                self.server.dispatch_bookkeeping_request(self.connection_index, next_exchange_index, MockWebServerSocket::new(self.raw.try_clone()?));
                next_exchange_index += 1;
                self.process_handshake_failure()?;
                return Ok(());
            }
            
            let addr = self.raw.peer_addr()?;
            let mut ssl_socket = factory.create_socket(self.raw.try_clone()?, addr.ip().to_string(), addr.port(), true)?;
            
            ssl_socket.set_use_client_mode(false)?;
            let auth = *self.server.client_auth.lock().unwrap();
            if auth == CLIENT_AUTH_REQUIRED {
                ssl_socket.set_need_client_auth(true)?;
            } else if auth == CLIENT_AUTH_REQUESTED {
                ssl_socket.set_want_client_auth(true)?;
            }

            ssl_socket.start_handshake()?;
            
            let protocol_str = ssl_socket.get_selected_protocol();
            protocol = protocol_str.and_then(|s| Protocol::get(&s).ok()).unwrap_or(Protocol::Http11);
            
            // Wrap the SSL socket in MockWebServerSocket (requires trait implementation)
            socket = MockWebServerSocket::new_ssl(ssl_socket);
        } else {
            protocol = if self.server.protocols.lock().unwrap().contains(&Protocol::H2PriorKnowledge) {
                Protocol::H2PriorKnowledge
            } else {
                Protocol::Http11
            };
            socket = MockWebServerSocket::new(self.raw.try_clone()?);
        }

        if let SocketEffect::Stall = self.first_exchange_peek.on_request_start {
            self.server.dispatch_bookkeeping_request(self.connection_index, next_exchange_index, socket);
            return Ok(());
        }

        if protocol == Protocol::Http2 || protocol == Protocol::H2PriorKnowledge {
            let mut h2_handler = Http2SocketHandler {
                server: Arc::clone(&self.server),
                connection_index: self.connection_index,
                socket,
                protocol,
                next_exchange_index: 0,
            };
            h2_handler.handle()?;
            return Ok(());
        }

        while self.process_one_request(&socket, &mut next_exchange_index)? {
            // continue
        }

        Ok(())
    }

    fn process_tunnel_requests(&self) -> io::Result<bool> {
        if !self.server.dispatcher.lock().unwrap().peek().in_tunnel { return Ok(true); }
        let socket = MockWebServerSocket::new(self.raw.try_clone()?);
        loop {
            let mut exch_idx = 0;
            if !self.process_one_request(&socket, &mut exch_idx)? {
                return Ok(false);
            }
            if !self.server.dispatcher.lock().unwrap().peek().in_tunnel { return Ok(true); }
        }
    }

    fn process_one_request(&self, socket: &MockWebServerSocket, exch_idx: &mut i32) -> io::Result<bool> {
        if socket.source.is_exhausted() { return Ok(false); }

        let request = self.read_request(socket, self.connection_index, *exch_idx)?;
        *exch_idx += 1;
        
        self.server.atomic_request_count.fetch_add(1, Ordering::SeqCst);
        self.server.request_queue.lock().unwrap().push(request.clone());

        if request.failure.is_some() { return Ok(false); }

        let response = self.server.dispatcher.lock().unwrap().dispatch(request);

        if self.handle_socket_effect(&response.on_response_start, socket) {
            return Ok(false);
        }

        let mut reuse_socket = true;
        let request_wants_socket = request.headers.get("Connection").map(|v| v.eq_ignore_ascii_case("Upgrade")).unwrap_or(false);
        let request_wants_websocket = request_wants_socket && request.headers.get("Upgrade").map(|v| v.eq_ignore_ascii_case("websocket")).unwrap_or(false);
        
        if request_wants_websocket && response.web_socket_listener.is_some() {
            self.handle_websocket_upgrade(socket, &request, &response)?;
            reuse_socket = false;
        } else {
            self.write_http_response(socket, &response)?;
        }

        if self.handle_socket_effect(&response.on_response_end, socket) {
            return Ok(false);
        }

        if response.shutdown_server {
            self.server.close();
        }

        Ok(reuse_socket)
    }

    fn read_request(&self, socket: &MockWebServerSocket, conn_idx: i32, exch_idx: i32) -> io::Result<RecordedRequest> {
        let mut headers_builder = Headers::builder();
        let mut content_length = -1i64;
        let mut chunked = false;
        let mut has_body = false;
        let mut body_buffer = Buffer::new();
        let mut received_bytes = 0i64;
        let mut chunk_sizes = Vec::new();

        let line = socket.source.read_utf8_line_strict()?;
        if line.is_empty() { return Err(IoError::new(ErrorKind::InvalidData, "no request")); }
        let request_line = decode_request_line(&line);

        loop {
            let header_line = socket.source.read_utf8_line_strict()?;
            if header_line.is_empty() { break; }
            headers_builder.add_line(&header_line);
            let lower = header_line.to_lowercase();
            if content_length == -1 && lower.starts_with("content-length:") {
                content_length = lower[15..].trim().parse().unwrap_or(-1);
            }
            if lower.starts_with("transfer-encoding:") && lower[18..].trim() == "chunked" {
                chunked = true;
            }
        }

        let peek = self.server.dispatcher.lock().unwrap().peek();
        for info in &peek.informational_responses {
            self.write_http_response(socket, info)?;
        }

        if !peek.do_not_read_request_body {
            if content_length != -1 && content_length > 0 {
                has_body = true;
                let bytes = socket.source.read_bytes(content_length as usize);
                body_buffer.write_all(&bytes);
                received_bytes += bytes.len() as i64;
            } else if chunked {
                has_body = true;
                loop {
                    let size_line = socket.source.read_utf8_line_strict()?;
                    let size = i32::from_str_radix(size_line.trim(), 16).unwrap_or(0);
                    if size == 0 {
                        socket.source.read_utf8_line_strict()?;
                        break;
                    }
                    chunk_sizes.push(size);
                    let bytes = socket.source.read_bytes(size as usize);
                    body_buffer.write_all(&bytes);
                    received_bytes += bytes.len() as i64;
                    socket.source.read_utf8_line_strict()?;
                }
            }
        }

        Ok(RecordedRequest {
            request_line,
            headers: headers_builder.build(),
            chunk_sizes: if chunked { Some(chunk_sizes) } else { None },
            body_size: received_bytes,
            body: if has_body { Some(body_buffer.read_byte_string()) } else { None },
            connection_index: conn_idx,
            exchange_index: exch_idx,
            socket: socket.clone(),
            failure: None,
        })
    }

    fn write_http_response(&self, socket: &MockWebServerSocket, response: &MockResponse) -> io::Result<()> {
        socket.sleep_while_open(response.headers_delay_nanos);
        socket.sink.write_all(response.status.as_bytes())?;
        socket.sink.write_all(b"\r\n")?;

        for (name, value) in &response.headers {
            socket.sink.write_all(name.as_bytes())?;
            socket.sink.write_all(b": ")?;
            socket.sink.write_all(value.as_bytes())?;
            socket.sink.write_all(b"\r\n")?;
        }
        socket.sink.write_all(b"\r\n")?;
        socket.sink.flush()?;

        if let Some(handler) = &response.socket_handler {
            handler.handle(socket);
            return Ok(());
        }

        if let Some(body) = &response.body {
            socket.sleep_while_open(response.body_delay_nanos);
            body.write_to(&mut socket.sink)?;
        }

        socket.sleep_while_open(response.trailers_delay_nanos);
        if response.headers.get("Transfer-Encoding").map(|v| v.eq_ignore_ascii_case("chunked")).unwrap_or(false) {
            for (name, value) in &response.trailers {
                socket.sink.write_all(name.as_bytes())?;
                socket.sink.write_all(b": ")?;
                socket.sink.write_all(value.as_bytes())?;
                socket.sink.write_all(b"\r\n")?;
            }
            socket.sink.write_all(b"\r\n")?;
        }

        Ok(())
    }

    fn handle_websocket_upgrade(&self, socket: &MockWebServerSocket, request: &RecordedRequest, response: &MockResponse) -> io::Result<()> {
        let key = request.headers.get("Sec-WebSocket-Key").expect("Missing key");
        let accept = WebSocketProtocol::accept_header(key);
        
        let mut ws_response = response.clone();
        ws_response.headers.add("Sec-WebSocket-Accept", &accept);
        self.write_http_response(socket, &ws_response)?;

        let scheme = "http";
        let authority = request.headers.get("Host").unwrap_or("localhost");
        let fancy_request = Request::builder()
            .url(format!("{}://{}/", scheme, authority))
            .headers(request.headers.clone())
            .build();

        let fancy_response = Response::builder()
            .code(ws_response.code)
            .message(ws_response.message)
            .headers(ws_response.headers.clone())
            .request(fancy_request)
            .protocol(Protocol::Http11)
            .build();

        let ws = RealWebSocket::new(
            Arc::clone(&self.server.task_runner),
            fancy_request,
            response.web_socket_listener.clone().expect("Listener required"),
            0,
            WebSocketExtensions::parse(ws_response.headers),
        );

        ws.init_reader_and_writer("MockWebServer WebSocket", socket, false);
        ws.loop_reader(fancy_response);
        socket.await_closed();
        Ok(())
    }

    fn handle_socket_effect(&self, effect: &Option<SocketEffect>, socket: &MockWebServerSocket) -> bool {
        match effect {
            Some(CloseStream { http2_error_code }) => {
                socket.close();
                true
            }
            Some(ShutdownConnection) => {
                socket.close();
                true
            }
            Some(CloseSocket { close_socket, shutdown_input, shutdown_output }) => {
                if *shutdown_input { socket.shutdown_input(); }
                if *shutdown_output { socket.shutdown_output(); }
                if *close_socket { socket.close(); }
                true
            }
            Some(Stall) => {
                socket.sleep_while_open(Duration::from_secs(3600).as_nanos() as i64);
                true
            }
            None => false,
        }
    }

    fn process_handshake_failure(&self) -> io::Result<()> {
        // Simplified: in a real impl, we'd use a real SSL context to force a failure
        Err(IoError::new(ErrorKind::ConnectionAborted, "Handshake failed"))
    }
}

struct Http2SocketHandler {
    server: Arc<MockWebServer>,
    connection_index: i32,
    socket: MockWebServerSocket,
    protocol: Protocol,
    next_exchange_index: i32,
}

impl Http2SocketHandler {
    fn handle(&mut self) -> io::Result<()> {
        let mut connection = Http2Connection::builder(false, Arc::clone(&self.server.task_runner))
            .socket(&self.socket, self.socket.remote_addr().to_string())
            .build();
        
        connection.start();
        
        // This is a simplified loop simulating the Http2Connection.Listener
        loop {
            let stream = connection.accept_stream()?;
            self.on_stream(stream)?;
        }
    }

    fn on_stream(&mut self, stream: Http2Stream) -> io::Result<()> {
        let peek = self.server.dispatcher.lock().unwrap().peek();
        if self.server.handle_socket_effect_h2(&peek.on_request_start, &self.socket, Some(&stream)) {
            self.server.dispatch_bookkeeping_request(self.connection_index, self.next_exchange_index, self.socket.clone());
            self.next_exchange_index += 1;
            return Ok(());
        }

        let request = self.read_request_h2(&stream)?;
        self.server.atomic_request_count.fetch_add(1, Ordering::SeqCst);
        self.server.request_queue.lock().unwrap().push(request.clone());

        if request.failure.is_some() { return Ok(()); }

        let response = self.server.dispatcher.lock().unwrap().dispatch(request);
        
        if self.server.handle_socket_effect_h2(&peek.on_response_start, &self.socket, Some(&stream)) {
            return Ok(());
        }

        self.write_response_h2(&stream, &response)?;
        self.server.handle_socket_effect_h2(&peek.on_response_end, &self.socket, Some(&stream));

        if response.shutdown_server {
            self.server.close();
        }

        Ok(())
    }

    fn read_request_h2(&mut self, stream: &Http2Stream) -> io::Result<RecordedRequest> {
        let stream_headers = stream.take_headers()?;
        let mut http_headers = Headers::builder();
        let mut method = "<:method omitted>".to_string();
        let mut path = "<:path omitted>".to_string();

        for (name, value) in stream_headers {
            if name == Header::TARGET_METHOD_UTF8 {
                method = value;
            } else if name == Header::TARGET_PATH_UTF8 {
                path = value;
            } else {
                http_headers.add(&name, &value);
            }
        }

        let request_line = RequestLine {
            method,
            target: path,
            version: "HTTP/2".to_string(),
        };

        let mut body_buffer = Buffer::new();
        let mut body_size = 0i64;
        
        // Read body from stream source
        let mut source = stream.source();
        let bytes = source.read_all();
        body_buffer.write_all(&bytes);
        body_size = bytes.len() as i64;

        Ok(RecordedRequest {
            request_line,
            headers: http_headers.build(),
            chunk_sizes: None,
            body_size,
            body: Some(body_buffer.read_byte_string()),
            connection_index: self.connection_index,
            exchange_index: self.next_exchange_index,
            socket: self.socket.clone(),
            failure: None,
        })
    }

    fn write_response_h2(&self, stream: &Http2Stream, response: &MockResponse) -> io::Result<()> {
        let mut h2_headers = Vec::new();
        h2_headers.push(Header::new(Header::RESPONSE_STATUS_UTF8.to_string(), response.code.to_string()));
        for (name, value) in &response.headers {
            h2_headers.push(Header::new(name.clone(), value.clone()));
        }

        stream.write_headers(h2_headers, response.body.is_none(), true)?;
        
        if let Some(body) = &response.body {
            body.write_to(&mut stream.sink)?;
        }
        
        Ok(())
    }
}

impl MockWebServer {
    fn handle_socket_effect_h2(&self, effect: &Option<SocketEffect>, socket: &MockWebServerSocket, stream: Option<&Http2Stream>) -> bool {
        match effect {
            Some(CloseStream { http2_error_code }) => {
                if let Some(s) = stream {
                    let _ = s.close(ErrorCode::from_http2(*http2_error_code).unwrap_or(ErrorCode::InternalError), None);
                } else {
                    socket.close();
                }
                true
            }
            Some(ShutdownConnection) => {
                if let Some(s) = stream {
                    let _ = s.connection.shutdown(ErrorCode::NoError);
                } else {
                    socket.close();
                }
                true
            }
            Some(CloseSocket { close_socket, shutdown_input, shutdown_output }) => {
                if *shutdown_input { socket.shutdown_input(); }
                if *shutdown_output { socket.shutdown_output(); }
                if *close_socket { socket.close(); }
                true
            }
            Some(Stall) => {
                socket.sleep_while_open(Duration::from_secs(3600).as_nanos() as i64);
                true
            }
            None => false,
        }
    }
}

impl std::fmt::Display for MockWebServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self.closed.lock().unwrap() {
            write!(f, "MockWebServer{{closed}}")
        } else {
            let addr = self.socket_address.lock().unwrap();
            match addr {
                Some(a) => write!(f, "MockWebServer{{port={}}}", a.port()),
                None => write!(f, "MockWebServer{{new}}"),
            }
        }
    }
}
)}
