use crate::okhttp3::{WebSocket, Response};
use okio::ByteString;
use std::error::Error;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

// Invoked when a web socket has been accepted by the remote peer and may begin transmitting
// messages.
pub trait WebSocketListener: Send + Sync {

    // Invoked when a text (type `0x1`) message has been received.

    // Invoked when a binary (type `0x2`) message has been received.

    // Invoked when the remote peer has indicated that no more incoming messages will be transmitted.

    // Invoked when both peers have indicated that no more messages will be transmitted and the
    // connection has been successfully released. No further calls to this listener will be made.

    // Invoked when a web socket has been closed due to an error reading from or writing to the
    // network. Both outgoing and incoming messages may have been lost. No further calls to this
    // listener will be made.

}
