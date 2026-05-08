/*
 * Copyright (C) 2025 Square, Inc.
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

/// An adverse action to take on a socket, intended to exercise failure modes in the calling code.
#[derive(Debug, Clone, PartialEq)]
pub enum SocketEffect {
    /// Close the TCP socket that carries this request.
    ///
    /// Using this as [MockResponse.on_response_end] is the default for HTTP/1.0.
    CloseSocket {
        close_socket: bool,
        shutdown_input: bool,
        shutdown_output: bool,
    },
    /// On HTTP/2, send a [GOAWAY frame](https://tools.ietf.org/html/rfc7540#section-6.8) immediately
    /// after the response and will close the connection when the client's socket is exhausted.
    ///
    /// On HTTP/1 this closes the socket.
    ShutdownConnection,
    /// On HTTP/2 this will send the error code on the stream.
    ///
    /// On HTTP/1 this closes the socket.
    CloseStream {
        http2_error_code: i32,
    },
    /// Stop processing this.
    Stall,
}

impl SocketEffect {
    /// Helper to create a CloseSocket effect with default values.
    pub fn close_socket(
        close_socket: bool,
        shutdown_input: bool,
        shutdown_output: bool,
    ) -> Self {
        SocketEffect::CloseSocket {
            close_socket,
            shutdown_input,
            shutdown_output,
        }
    }

    /// Helper to create a CloseStream effect with a specific error code.
    pub fn close_stream(http2_error_code: i32) -> Self {
        SocketEffect::CloseStream { http2_error_code }
    }
}

impl Default for SocketEffect {
    fn default() -> Self {
        // Default behavior for CloseSocket as defined in Kotlin:
        // closeSocket = true, shutdownInput = false, shutdownOutput = false
        SocketEffect::CloseSocket {
            close_socket: true,
            shutdown_input: false,
            shutdown_output: false,
        }
    }
}