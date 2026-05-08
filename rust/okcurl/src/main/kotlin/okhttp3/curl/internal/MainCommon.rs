/*
 * Copyright (C) 2021 Square, Inc.
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

use std::io::{self, Write};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody;
use crate::okcurl::src::main::kotlin::okhttp3::curl::Main;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::StatusLine;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okcurl::src::main::kotlin::okhttp3::curl::MainCommandLine::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::internal::http::StatusLineTest::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::JsseDebugLogging::*;

// Internal extension to Main to create a Request based on current CLI arguments.
pub trait MainCommonRequest {
    fn common_create_request(&self) -> Result<Request, Box<dyn std::error::Error>>;
    fn media_type(&self) -> Option<MediaType>;
}

impl MainCommonRequest for Main {
    fn common_create_request(&self) -> Result<Request, Box<dyn std::error::Error>> {
        let mut builder = Request::builder();

        // val requestMethod = method ?: if (data != null) "POST" else "GET"
        let request_method = self.method.as_ref().map(|s| s.as_str()).unwrap_or_else(|| {
            if self.data.is_some() {
                "POST"
            } else {
                "GET"
            }
        });

        // val url = url ?: throw IOException("No url provided")
        let url = self.url.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "No url provided")
        })?;

        builder.url(url);

        // data?.r#let { request.method(requestMethod, it.toRequestBody(mediaType())) }
        if let Some(ref data_content) = self.data {
            let media_type = self.media_type();
            // Assuming RequestBody::to_request_body is the Rust equivalent of the Kotlin extension
            let body = RequestBody::to_request_body(data_content, media_type);
            builder.method(request_method.to_string(), Some(body));
        }

        // for (header in headers.orEmpty()) { ... }
        if let Some(ref headers) = self.headers {
            for header in headers {
                let parts: Vec<&str> = header.splitn(2, ':').collect();
                if parts.len() == 2 {
                    if !is_special_header(parts[0]) {
                        builder.header(parts[0], parts[1]);
                    }
                }
            }
        }

        // referer?.r#let { request.header("Referer", it) }
        if let Some(ref referer) = self.referer {
            builder.header("Referer", referer);
        }

        builder.header("User-Agent", &self.user_agent);

        Ok(builder.build())
    }

    fn media_type(&self) -> Option<MediaType> {
        let mime_type = if let Some(ref headers) = self.headers {
            let mut found_mime = None;
            for header in headers {
                let parts: Vec<&str> = header.splitn(2, ':').collect();
                if parts.len() == 2 && parts[0].eq_ignore_ascii_case("Content-Type") {
                    found_mime = Some(parts[1].trim().to_string());
                    break;
                }
            }
            found_mime
        } else {
            Some("application/x-www-form-urlencoded".to_string())
        };

        mime_type.and_then(|mt| MediaType::to_media_type_or_null(&mt))
    }
}

fn is_special_header(s: &str) -> bool {
    s.eq_ignore_ascii_case("Content-Type")
}

pub trait MainCommonRun {
    fn common_run(&mut self);
}

impl MainCommonRun for Main {
    fn common_run(&mut self) {
        // client = createClient()
        self.client = Some(self.create_client());
        
        // val request = createRequest()
        let request_result = self.common_create_request();
        
        if let Ok(request) = request_result {
            let client_ref = self.client.as_ref().expect("Client should be initialized");
            
            // try { ... } catch (e: IOException) { ... } finally { close() }
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let response = client_ref.new_call(request).execute()?;
                
                if self.show_headers {
                    println!("{}", StatusLine::get(&response));
                    for (name, value) in response.headers().iter() {
                        println!("{}: {}", name, value);
                    }
                    println!();
                }

                // Stream the response to the System.out as it is returned from the server.
                let mut out = io::stdout();
                let mut source = response.body().unwrap().source();
                
                while !source.exhausted() {
                    let buffer = source.buffer();
                    let size = buffer.size();
                    out.write_all(buffer.as_bytes())?;
                    out.flush()?;
                }

                response.body().unwrap().close();
                Ok(())
            })();

            if let Err(e) = result {
                eprintln!("IOException: {}", e);
            }
        }

        // finally { close() }
        self.close();
    }
}
