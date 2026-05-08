use std::io::{Read, Result as IoResult};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;

// Note: Header and ErrorCode are assumed to be defined in the same module (okhttp3.internal.http2)
// as they are used in the PushObserver interface.
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::{Header, ErrorCode};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;

// Note: BufferedSource is a specific Okio type. In Rust, this is represented as a type 
// that implements std::io::Read.
pub trait PushObserver: Send + Sync {
    /*
     * Describes the request that the server intends to push a response for.
     *
     * @param stream_id server-initiated stream ID: an even number.
     * @param request_headers minimally includes `:method`, `:scheme`, `:authority`,
     * and `:path`.
     */
    fn on_request(
        &self,
        stream_id: i32,
        request_headers: Vec<Header>,
    ) -> bool;

    /*
     * The response headers corresponding to a pushed request. When `last` is true, there are
     * no data frames to follow.
     *
     * @param stream_id server-initiated stream ID: an even number.
     * @param response_headers minimally includes `:status`.
     * @param last when true, there is no response data.
     */
    fn on_headers(
        &self,
        stream_id: i32,
        response_headers: Vec<Header>,
        last: bool,
    ) -> bool;

    /*
     * A chunk of response data corresponding to a pushed request. This data must either be read or
     * skipped.
     *
     * @param stream_id server-initiated stream ID: an even number.
     * @param source location of data corresponding with this stream ID.
     * @param byte_count number of bytes to read or skip from the source.
     * @param last when true, there are no data frames to follow.
     */
    fn on_data(
        &self,
        stream_id: i32,
        source: &mut dyn Read,
        byte_count: i32,
        last: bool,
    ) -> IoResult<bool>;

    /* Indicates the reason why this stream was canceled. */
    fn on_reset(
        &self,
        stream_id: i32,
        error_code: ErrorCode,
    );
}

// Implementation of PushObserver that cancels all pushed streams.
#[derive(Debug, Clone, PartialEq)]
struct PushObserverCancel;

impl PushObserver for PushObserverCancel {
    fn on_request(
        &self,
        _stream_id: i32,
        _request_headers: Vec<Header>,
    ) -> bool {
        true
    }

    fn on_headers(
        &self,
        _stream_id: i32,
        _response_headers: Vec<Header>,
        _last: bool,
    ) -> bool {
        true
    }

    fn on_data(
        &self,
        _stream_id: i32,
        source: &mut dyn Read,
        byte_count: i32,
        _last: bool,
    ) -> IoResult<bool> {
        // In Rust, skipping bytes from a Read source is typically done by reading into a dummy buffer
        // or using Take. To preserve the behavior of Okio's BufferedSource.skip():
        let mut buffer = vec![0u8; byte_count as usize];
        source.read_exact(&mut buffer)?;
        Ok(true)
    }

}

// Companion object equivalent for PushObserver.
pub struct PushObserverCompanion;

impl PushObserverCompanion {
    // A PushObserver that cancels all pushed streams.
    pub fn cancel() -> Box<dyn PushObserver> {
        Box::new(PushObserverCancel)
    }
}

// To match the Kotlin `val CANCEL: PushObserver = PushObserverCancel()`, 
// we provide a way to access a singleton-like instance.
pub static CANCEL: std::sync::OnceLock<Box<dyn PushObserver>> = std::sync::OnceLock::new();

pub fn get_cancel_observer() -> &'static Box<dyn PushObserver> {
    CANCEL.get_or_init(|| Box::new(PushObserverCancel))
}