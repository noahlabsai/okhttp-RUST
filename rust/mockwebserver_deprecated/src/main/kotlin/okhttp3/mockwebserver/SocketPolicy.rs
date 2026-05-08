/*
 * Copyright (C) 2020 Square, Inc.
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

use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketPolicy {
    ShutdownServerAfterResponse,
    KeepOpen,
    DisconnectAtEnd,
    UpgradeToSslAtEnd,
    DisconnectAtStart,
    DisconnectAfterRequest,
    DisconnectDuringRequestBody,
    DisconnectDuringResponseBody,
    DoNotReadRequestBody,
    FailHandshake,
    ShutdownInputAtEnd,
    ShutdownOutputAtEnd,
    StallSocketAtStart,
    NoResponse,
    ResetStreamAtStart,
    ExpectContinue,
    ContinueAlways,
}

impl Default for SocketPolicy {
    fn default() -> Self {
        SocketPolicy::ShutdownServerAfterResponse
    }
}

pub const ShutdownServerAfterResponse: SocketPolicy = SocketPolicy::ShutdownServerAfterResponse;
pub const KeepOpen: SocketPolicy = SocketPolicy::KeepOpen;
pub const DisconnectAtEnd: SocketPolicy = SocketPolicy::DisconnectAtEnd;
pub const UpgradeToSslAtEnd: SocketPolicy = SocketPolicy::UpgradeToSslAtEnd;
pub const DisconnectAtStart: SocketPolicy = SocketPolicy::DisconnectAtStart;
pub const DisconnectAfterRequest: SocketPolicy = SocketPolicy::DisconnectAfterRequest;
pub const DisconnectDuringRequestBody: SocketPolicy = SocketPolicy::DisconnectDuringRequestBody;
pub const DisconnectDuringResponseBody: SocketPolicy = SocketPolicy::DisconnectDuringResponseBody;
pub const DoNotReadRequestBody: SocketPolicy = SocketPolicy::DoNotReadRequestBody;
pub const FailHandshake: SocketPolicy = SocketPolicy::FailHandshake;
pub const ShutdownInputAtEnd: SocketPolicy = SocketPolicy::ShutdownInputAtEnd;
pub const ShutdownOutputAtEnd: SocketPolicy = SocketPolicy::ShutdownOutputAtEnd;
pub const StallSocketAtStart: SocketPolicy = SocketPolicy::StallSocketAtStart;
pub const NoResponse: SocketPolicy = SocketPolicy::NoResponse;
pub const ResetStreamAtStart: SocketPolicy = SocketPolicy::ResetStreamAtStart;
pub const ExpectContinue: SocketPolicy = SocketPolicy::ExpectContinue;
pub const ContinueAlways: SocketPolicy = SocketPolicy::ContinueAlways;