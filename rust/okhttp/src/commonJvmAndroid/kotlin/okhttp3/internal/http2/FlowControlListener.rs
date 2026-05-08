/*
 * Copyright (C) 2023 Block, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::flowcontrol::WindowCounter;

/// Interface for listening to flow control window changes.
pub trait FlowControlListener: Send + Sync {
    /// Notification that the receiving stream flow control window has changed.
    /// `WindowCounter` generally carries the client view of total and acked bytes.
    fn receiving_stream_window_changed(
        &self,
        stream_id: i32,
        window_counter: &WindowCounter,
        buffer_size: i64,
    );

    /// Notification that the receiving connection flow control window has changed.
    /// `WindowCounter` generally carries the client view of total and acked bytes.
    fn receiving_connection_window_changed(&self, window_counter: &WindowCounter);
}

/// Noop implementation of FlowControlListener.
pub struct None;

impl FlowControlListener for None {
    fn receiving_stream_window_changed(
        &self,
        _stream_id: i32,
        _window_counter: &WindowCounter,
        _buffer_size: i64,
    ) {
        // Noop implementation
    }

    fn receiving_connection_window_changed(&self, _window_counter: &WindowCounter) {
        // Noop implementation
    }
}