use std::sync::{Arc, Mutex};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::SocketAdapter;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;

// Mocking SSLSocket as it is a platform type from javax.net.ssl
// In a real production environment, this would be a binding to the Android/JVM SSLSocket.
pub struct SSLSocket;

// Factory for creating SocketAdapter instances.
pub trait Factory: Send + Sync {
    fn matches_socket(&self, ssl_socket: &SSLSocket) -> bool;
    fn create(&self, ssl_socket: &SSLSocket) -> Arc<dyn SocketAdapter>;
}

/*
 * Deferred implementation of SocketAdapter that works by observing the socket
 * and initializing on first use.
 *
 * We use this because eager classpath checks cause confusion and excessive logging in Android,
 * and we can't rely on classnames after proguard, so are probably best served by falling through
 * to a situation of trying our least likely noisiest options.
 */
pub struct DeferredSocketAdapter {
    socket_adapter_factory: Box<dyn Factory>,
    delegate: Mutex<Option<Arc<dyn SocketAdapter>>>,
}

impl DeferredSocketAdapter {
    pub fn new(socket_adapter_factory: Box<dyn Factory>) -> Self {
        Self {
            socket_adapter_factory,
            delegate: Mutex::new(None),
        }
    }

    // @Synchronized private fun getDelegate(sslSocket: SSLSocket): SocketAdapter?
    fn get_delegate(&self, ssl_socket: &SSLSocket) -> Option<Arc<dyn SocketAdapter>> {
        let mut delegate_lock = self.delegate.lock().unwrap();
        
        if delegate_lock.is_none() && self.socket_adapter_factory.matches_socket(ssl_socket) {
            *delegate_lock = Some(self.socket_adapter_factory.create(ssl_socket));
        }

        delegate_lock.clone()
    }
}

impl SocketAdapter for DeferredSocketAdapter {
    fn is_supported(&self) -> bool {
        true
    }

    fn matches_socket(&self, ssl_socket: &SSLSocket) -> bool {
        self.socket_adapter_factory.matches_socket(ssl_socket)
    }

    fn configure_tls_extensions(
        &self,
        ssl_socket: &mut SSLSocket,
        hostname: Option<String>,
        protocols: Vec<Protocol>,
    ) {
        if let Some(delegate) = self.get_delegate(ssl_socket) {
            delegate.configure_tls_extensions(ssl_socket, hostname, protocols);
        }
    }

    fn get_selected_protocol(&self, ssl_socket: &SSLSocket) -> Option<String> {
        self.get_delegate(ssl_socket)
            .and_then(|delegate| delegate.get_selected_protocol(ssl_socket))
    }
}