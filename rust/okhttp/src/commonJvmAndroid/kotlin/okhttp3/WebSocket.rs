use std::error::Error;
use crate::okhttp3::{Request, WebSocketListener};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketHandler::*;

// A non-blocking interface to a web socket. Use the [WebSocket::Factory] to create
// instances; usually this is [OkHttpClient].
//
// ## Web Socket Lifecycle
//
// Upon normal operation each web socket progresses through a sequence of states:
//
//  * **Connecting:** the initial state of each web socket. Messages may be enqueued but they won't
//    be transmitted until the web socket is open.
//
//  * **Open:** the web socket has been accepted by the remote peer and is fully operational.
//    Messages in either direction are enqueued for immediate transmission.
//
//  * **Closing:** one of the peers on the web socket has initiated a graceful shutdown. The web
//    socket will continue to transmit already-enqueued messages but will refuse to enqueue new
//    ones.
//
//  * **Closed:** the web socket has transmitted all of its messages and has received all messages
//    from the peer.
//
// Web sockets may fail due to HTTP upgrade problems, connectivity problems, or if either peer
// chooses to short-circuit the graceful shutdown process:
//
//  * **Canceled:** the web socket connection failed. Messages that were successfully enqueued by
//    either peer may not have been transmitted to the other.
//
// Note that the state progression is independent for each peer. Arriving at a gracefully-closed
// state indicates that a peer has sent all of its outgoing messages and received all of its
// incoming messages. But it does not guarantee that the other peer will successfully receive all of
// its incoming messages.
//
// ## Message Queue
//
// Messages enqueued with [send_text] or [send_bytes] are buffered in an outgoing message queue. This queue has a 16 MiB
// limit. If a call to [send] would cause the queue to exceed this limit, the web socket will
// initiate a graceful shutdown (close code 1001) and `send()` will return `false`. No exception is
// thrown and no [WebSocketListener::on_failure] callback is triggered, so callers should always check
// the return value of `send()`.
//
// Use [queue_size] to monitor backpressure before sending. For large payloads, consider breaking
// them into smaller messages or using HTTP requests instead.
pub trait WebSocket: Send + Sync {
    // Returns the original request that initiated this web socket.
    fn request(&self) -> Request;

    // Returns the size in bytes of all messages enqueued to be transmitted to the server. This
    // doesn't include framing overhead. If compression is enabled, uncompressed messages size
    // is used to calculate this value. It also doesn't include any bytes buffered by the operating
    // system or network intermediaries. This method returns 0 if no messages are waiting in the
    // queue. If may return a nonzero value after the web socket has been canceled; this indicates
    // that enqueued messages were not transmitted.
    //
    // Use this to monitor backpressure and avoid exceeding the 16 MiB outgoing message buffer limit.
    // When that limit is exceeded, the web socket is gracefully shut down.
    fn queue_size(&self) -> i64;

    // Attempts to enqueue `text` to be UTF-8 encoded and sent as a the data of a text (type `0x1`)
    // message.
    //
    // This method returns true if the message was enqueued. Messages that would overflow the outgoing
    // message buffer (16 MiB) will be rejected and trigger a [graceful shutdown][close] of this web
    // socket. This method returns false in that case, and in any other case where this web socket is
    // closing, closed, or canceled.
    //
    // This method returns immediately.
    fn send_text(&self, text: String) -> bool;

    // Attempts to enqueue `bytes` to be sent as a the data of a binary (type `0x2`) message.
    //
    // This method returns true if the message was enqueued. Messages that would overflow the outgoing
    // message buffer (16 MiB) will be rejected and trigger a [graceful shutdown][close] of this web
    // socket. This method returns false in that case, and in any other case where this web socket is
    // closing, closed, or canceled.
    //
    // This method returns immediately.
    fn send_bytes(&self, bytes: Vec<u8>) -> bool;

    // Attempts to initiate a graceful shutdown of this web socket. Any already-enqueued messages will
    // be transmitted before the close message is sent but subsequent calls to [send] will return
    // false and their messages will not be enqueued.
    //
    // This returns true if a graceful shutdown was initiated by this call. It returns false if
    // a graceful shutdown was already underway or if the web socket is already closed or canceled.
    //
    // @param code Status code as defined by
    //     [Section 7.4 of RFC 6455](http://tools.ietf.org/html/rfc6455#section-7.4).
    // @param reason Reason for shutting down, no longer than 123 bytes of UTF-8 encoded data (**not** characters) or null.
    // @throws IllegalArgumentException if [code] is invalid or [reason] is too long.
    fn close(&self, code: i32, reason: Option<String>) -> Result<bool, Box<dyn Error + Send + Sync>>;

    // Immediately and violently release resources held by this web socket, discarding any enqueued
    // messages. This does nothing if the web socket has already been closed or canceled.
    fn cancel(&self);
}

pub trait Factory: Send + Sync {
    // Creates a new web socket and immediately returns it. Creating a web socket initiates an
    // asynchronous process to connect the socket. Once that succeeds or fails, `listener` will be
    // notified. The caller must either close or cancel the returned web socket when it is no longer
    // in use.
    fn new_web_socket(
        &self,
        request: Request,
        listener: WebSocketListener,
    ) -> Box<dyn WebSocket>;
}