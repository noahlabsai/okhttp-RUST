use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::SuppressSignatureCheck;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Call, Connection, Route};
use std::io::Error as IOException;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;

// Data classes that correspond to each of the methods of [ConnectionListener].
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionEvent {
    ConnectStart {
        timestamp_ns: i64,
        route: Route,
        call: Call,
    }

impl Default for ConnectionEvent {
    fn default() -> Self {
        ConnectionEvent::ConnectStart
    }
}

pub const ConnectStart: ConnectionEvent = ConnectionEvent::ConnectStart;
pub const timestamp_ns: ConnectionEvent = ConnectionEvent::timestamp_ns;
pub const route: ConnectionEvent = ConnectionEvent::route;
pub const call: ConnectionEvent = ConnectionEvent::call;,
    ConnectFailed {
        timestamp_ns: i64,
        route: Route,
        call: Call,
        exception: IOException,
    },
    ConnectEnd {
        timestamp_ns: i64,
        connection: Connection,
        route: Route,
        call: Call,
    },
    ConnectionClosed {
        timestamp_ns: i64,
        connection: Connection,
    },
    ConnectionAcquired {
        timestamp_ns: i64,
        connection: Connection,
        call: Call,
    },
    ConnectionReleased {
        timestamp_ns: i64,
        connection: Connection,
        call: Call,
    },
    NoNewExchanges {
        timestamp_ns: i64,
        connection: Connection,
    },
}

impl ConnectionEvent {
    // Returns the timestamp of the event in nanoseconds.
    pub fn timestamp_ns(&self) -> i64 {
        match self {
            ConnectionEvent::ConnectStart { timestamp_ns, .. } => *timestamp_ns,
            ConnectionEvent::ConnectFailed { timestamp_ns, .. } => *timestamp_ns,
            ConnectionEvent::ConnectEnd { timestamp_ns, .. } => *timestamp_ns,
            ConnectionEvent::ConnectionClosed { timestamp_ns, .. } => *timestamp_ns,
            ConnectionEvent::ConnectionAcquired { timestamp_ns, .. } => *timestamp_ns,
            ConnectionEvent::ConnectionReleased { timestamp_ns, .. } => *timestamp_ns,
            ConnectionEvent::NoNewExchanges { timestamp_ns, .. } => *timestamp_ns,
        }
    }

    // Returns the connection associated with this event, or None if there is no connection.
    pub fn connection(&self) -> Option<&Connection> {
        match self {
            ConnectionEvent::ConnectEnd { connection, .. } => Some(connection),
            ConnectionEvent::ConnectionClosed { connection, .. } => Some(connection),
            ConnectionEvent::ConnectionAcquired { connection, .. } => Some(connection),
            ConnectionEvent::ConnectionReleased { connection, .. } => Some(connection),
            ConnectionEvent::NoNewExchanges { connection, .. } => Some(connection),
            _ => None,
        }
    }

    // Returns if the event closes this event, or None if this is no open event.
    pub fn closes(&self, event: &ConnectionEvent) -> Option<bool> {
        match self {
            ConnectionEvent::ConnectFailed { route, call, .. } => {
                if let ConnectionEvent::ConnectStart { route: e_route, call: e_call, .. } = event {
                    Some(call == e_call && route == e_route)
                } else {
                    Some(false)
                }
            }
            ConnectionEvent::ConnectEnd { route, call, .. } => {
                if let ConnectionEvent::ConnectStart { route: e_route, call: e_call, .. } = event {
                    Some(call == e_call && route == e_route)
                } else {
                    Some(false)
                }
            }
            ConnectionEvent::ConnectionReleased { connection, call, .. } => {
                if let ConnectionEvent::ConnectionAcquired { connection: e_conn, call: e_call, .. } = event {
                    Some(connection == e_conn && call == e_call)
                } else {
                    Some(false)
                }
            }
            _ => None,
        }
    }

    // Returns the simple name of the event class.
    pub fn name(&self) -> String {
        match self {
            ConnectionEvent::ConnectStart { .. } => "ConnectStart".to_string(),
            ConnectionEvent::ConnectFailed { .. } => "ConnectFailed".to_string(),
            ConnectionEvent::ConnectEnd { .. } => "ConnectEnd".to_string(),
            ConnectionEvent::ConnectionClosed { .. } => "ConnectionClosed".to_string(),
            ConnectionEvent::ConnectionAcquired { .. } => "ConnectionAcquired".to_string(),
            ConnectionEvent::ConnectionReleased { .. } => "ConnectionReleased".to_string(),
            ConnectionEvent::NoNewExchanges { .. } => "NoNewExchanges".to_string(),
        }
    }
}