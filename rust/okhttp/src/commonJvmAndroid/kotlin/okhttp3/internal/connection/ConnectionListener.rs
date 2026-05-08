use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Call::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Connection::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Route::*;
use std::error::Error;
use std::sync::Arc;

// Listener for connection events. Extend this class to monitor the new connections and closes.
//
// All event methods must execute fast, without external locking, cannot throw exceptions,
// attempt to mutate the event parameters, or be reentrant back into the client.
// Any IO - writing to files or network should be done asynchronously.
pub trait ConnectionListener: Send + Sync {
    // Invoked as soon as a call causes a connection to be started.

    // Invoked when a connection fails to be established.
    fn connect_failed(&self, route: &Route, call: &dyn Call, failure: &(dyn Error + Send + Sync)) {
        // Default implementation: do nothing
    }

    // Invoked as soon as a connection is successfully established.

    // Invoked when a connection is released as no longer required.

    // Invoked when a call is assigned a particular connection.

    // Invoked when a call no longer uses a connection.

    // Invoked when a connection is marked for no new exchanges.

}

// Implementation of ConnectionListener that does nothing.
// Equivalent to the Kotlin `object : ConnectionListener() {}` used in the companion object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoOpConnectionListener;

impl ConnectionListener for NoOpConnectionListener {
    fn connect_start(&self, _route: &Route, _call: &dyn Call) {}
    fn connect_failed(&self, _route: &Route, _call: &dyn Call, _failure: &(dyn Error + Send + Sync)) {}
    fn connect_end(&self, _connection: &dyn Connection, _route: &Route, _call: &dyn Call) {}
    fn connection_closed(&self, _connection: &dyn Connection) {}
    fn connection_acquired(&self, _connection: &dyn Connection, _call: &dyn Call) {}
    fn connection_released(&self, _connection: &dyn Connection, _call: &dyn Call) {}
    fn no_new_exchanges(&self, _connection: &dyn Connection) {}
}

// Companion object equivalent for ConnectionListener.
pub struct ConnectionListenerCompanion;

impl ConnectionListenerCompanion {
    // The NONE listener that does nothing.
    pub fn none() -> Arc<dyn ConnectionListener> {
        Arc::new(NoOpConnectionListener)
    }
}