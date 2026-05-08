use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Import paths as specified in the translation rules
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::TaskRunner::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Header::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Connection::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Stream::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::Platform::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ProtocolTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;

// Mocking SSLSocketFactory and SSLSocket as they are Java-specific.
// In a real Rust production environment, these would be replaced by `rustls` or `native-tls`.
pub trait SslSocketFactory {
    fn create_socket(&self, socket: TcpStream, host: String, port: u16, auto_close: bool) -> Box<dyn SslSocket>;
}

pub trait SslSocket: Read + Write + Send {
    fn set_use_client_mode(&mut self, use_client_mode: bool);
    fn start_handshake(&mut self) -> io::Result<()>;
    fn peer_name(&self) -> String;
    fn get_tcp_stream(&self) -> &TcpStream;
}

// Extension trait to mimic Kotlin's asBufferedSocket() and peerName()
trait SocketExt {
    fn peer_name_ext(&self) -> String;
}

impl SocketExt for TcpStream {
    fn peer_name_ext(&self) -> String {
        match self.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(_) => "unknown".to_string(),
        }
    }
}

// A basic HTTP/2 server that serves the contents of a local directory.
pub struct Http2Server {
    base_directory: PathBuf,
    ssl_socket_factory: Arc<dyn SslSocketFactory>,
}

impl Http2Server {
    pub fn new(base_directory: PathBuf, ssl_socket_factory: Arc<dyn SslSocketFactory>) -> Self {
        Self {
            base_directory,
            ssl_socket_factory,
        }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind("0.0.0.0:8888").expect("Could not bind to port 8888");
        // Note: reuse_address is typically handled by the OS or specific socket crates in Rust
        
        loop {
            match listener.accept() {
                Ok((socket, _)) => {
                    if let Err(e) = self.handle_connection(socket) {
                        eprintln!("Http2Server connection failure: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Http2Server accept failure: {}", e);
                }
            }
        }
    }

    fn handle_connection(&self, socket: TcpStream) -> io::Result<()> {
        let mut ssl_socket = self.do_ssl(socket)?;
        
        // Platform.get().getSelectedProtocol(sslSocket)
        let protocol_string = Platform::get().get_selected_protocol(&*ssl_socket);
        
        let protocol = protocol_string.and_then(|s| Protocol::get(&s).ok());
        
        if protocol != Some(Protocol::Http2) {
            return Err(io::Error::new(io::ErrorKind::Other, "Protocol unsupported"));
        }

        // Http2Connection.Builder(false, TaskRunner.INSTANCE)
        let connection = Http2ConnectionBuilder::new(false, TaskRunner::get_instance())
            .socket(ssl_socket.get_tcp_stream(), ssl_socket.peer_name())
            .listener(self)
            .build();
            
        connection.start();
        Ok(())
    }

    fn do_ssl(&self, socket: TcpStream) -> io::Result<Box<dyn SslSocket>> {
        let addr = socket.peer_addr()?;
        let host = addr.ip().to_string();
        let port = addr.port();

        let mut ssl_socket = self.ssl_socket_factory.create_socket(socket, host, port, true);
        ssl_socket.set_use_client_mode(false);
        
        // Platform.get().configureTlsExtensions(sslSocket, null, listOf(Protocol.HTTP_2))
        Platform::get().configure_tls_extensions(
            ssl_socket.as_mut(), 
            None, 
            vec![Protocol::Http2]
        );
        
        ssl_socket.start_handshake()?;
        Ok(ssl_socket)
    }

    fn content_type(&self, path: &Path) -> String {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.ends_with(".css") { "text/css".to_string() }
        else if name.ends_with(".gif") { "image/gif".to_string() }
        else if name.ends_with(".html") { "text/html".to_string() }
        else if name.ends_with(".jpeg") { "image/jpeg".to_string() }
        else if name.ends_with(".jpg") { "image/jpeg".to_string() }
        else if name.ends_with(".js") { "application/javascript".to_string() }
        else if name.ends_with(".png") { "image/png".to_string() }
        else { "text/plain".to_string() }
    }

    fn send_404(&self, stream: &Http2Stream, path: &str) -> io::Result<()> {
        let response_headers = vec![
            Header::from_strings(":status", "404"),
            Header::from_strings(":version", "HTTP/1.1"),
            Header::from_strings("content-type", "text/plain"),
        ];
        
        stream.write_headers(response_headers, false, false)?;
        
        let mut sink = stream.sink().buffer();
        sink.write_all(format!("Not found: {}", path).as_bytes())?;
        sink.flush()?;
        Ok(())
    }

    fn serve_directory(&self, stream: &Http2Stream, files: Vec<fs::ReadDir>) -> io::Result<()> {
        let response_headers = vec![
            Header::from_strings(":status", "200"),
            Header::from_strings(":version", "HTTP/1.1"),
            Header::from_strings("content-type", "text/html; charset=UTF-8"),
        ];
        
        stream.write_headers(response_headers, false, false)?;
        
        let mut sink = stream.sink().buffer();
        for entry in files.into_iter().flatten().flatten() {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            let target = if entry.path().is_dir() {
                format!("{}/", file_name)
            } else {
                file_name
            };
            sink.write_all(format!("<a href='{}'>{}</a><br>", target, target).as_bytes())?;
        }
        sink.flush()?;
        Ok(())
    }

    fn serve_file(&self, stream: &Http2Stream, file_path: PathBuf) -> io::Result<()> {
        let response_headers = vec![
            Header::from_strings(":status", "200"),
            Header::from_strings(":version", "HTTP/1.1"),
            Header::from_strings("content-type", &self.content_type(&file_path)),
        ];
        
        stream.write_headers(response_headers, false, false)?;
        
        let mut file = File::open(file_path)?;
        let mut sink = stream.sink().buffer();
        io::copy(&mut file, &mut sink)?;
        sink.flush()?;
        Ok(())
    }
}

impl Http2ConnectionListener for Http2Server {
    fn on_stream(&self, stream: Http2Stream) {
        let result = (|| -> io::Result<()> {
            let request_headers = stream.take_headers()?;
            let mut path: Option<String> = None;
            
            for i in 0..request_headers.size() {
                if request_headers.name(i) == Header::TARGET_PATH_UTF8 {
                    path = Some(request_headers.value(i));
                    break;
                }
            }
            
            let path = path.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing path"))?;
            
            let mut full_path = self.base_directory.clone();
            full_path.push(&path);
            
            if full_path.is_dir() {
                let entries = fs::read_dir(&full_path)?;
                self.serve_directory(&stream, vec![entries])?;
            } else if full_path.exists() {
                self.serve_file(&stream, full_path)?;
            } else {
                self.send_404(&stream, &path)?;
            }
            Ok(())
        })();

        if let Err(e) = result {
            Platform::get().log(
                &format!("Failure serving Http2Stream: {}", e),
                PlatformLogLevel::Info,
                None
            );
        }
    }
}

impl Http2Server {
    pub fn main(args: Vec<String>) {
        if args.len() != 2 || args[1].starts_with('-') {
            println!("Usage: Http2Server <base directory>");
            return;
        }
        
        // In a real scenario, localhost().sslContext().socketFactory would be provided
        // Here we assume a mock implementation of SslSocketFactory
        let socket_factory = Arc::new(MockSslSocketFactory {}); 
        let server = Http2Server::new(PathBuf::from(&args[1]), socket_factory);
        server.run();
    }
}

// Mock implementation for compilation
struct MockSslSocketFactory;
impl SslSocketFactory for MockSslSocketFactory {
    fn create_socket(&self, socket: TcpStream, _host: String, _port: u16, _auto_close: bool) -> Box<dyn SslSocket> {
        Box::new(MockSslSocket { socket })
    }
}

struct MockSslSocket {
    socket: TcpStream,
}

impl Read for MockSslSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.socket.read(buf) }
}

impl Write for MockSslSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.socket.write(buf) }
    fn flush(&mut self) -> io::Result<()> { self.socket.flush() }
}

impl SslSocket for MockSslSocket {
    fn set_use_client_mode(&mut self, _use_client_mode: bool) {}
    fn start_handshake(&mut self) -> io::Result<()> { Ok(()) }
    fn peer_name(&self) -> String { self.socket.peer_name_ext() }
    fn get_tcp_stream(&self) -> &TcpStream { &self.socket }
}