use okio::{Buffer, BufferedSource, ByteString, Options};
use std::io::IOException;
use crate::android_test::build_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::UtilCommon::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;

pub struct ServerSentEventReader<C: Callback> {
    source: BufferedSource,
    callback: C,
    last_id: Option<String>,
}

pub trait Callback {
    fn on_event(&self, id: Option<String>, r#type: Option<String>, data: String);
    fn on_retry_change(&self, time_ms: i64);
}

impl<C: Callback> ServerSentEventReader<C> {
    pub fn new(source: BufferedSource, callback: C) -> Self {
        Self {
            source,
            callback,
            last_id: None,
        }
    }

    // Process the next event. This will result in a single call to [Callback.on_event] *unless* the
    // data section was empty. Any number of calls to [Callback.on_retry_change] may occur while
    // processing an event.
    //
    // @return false when EOF is reached
    pub fn process_next_event(&mut self) -> Result<bool, IOException> {
        let mut id = self.last_id.clone();
        let mut r#type: Option<String> = None;
        let mut data = Buffer::new();

        loop {
            match self.source.select(&OPTIONS) {
                0..=2 => {
                    self.complete_event(id, r#type, data)?;
                    return Ok(true);
                }
                3..=4 => {
                    self.read_data(&mut data)?;
                }
                5..=7 => {
                    data.write_byte(b'\n'); // 'data' on a line of its own.
                }
                8..=9 => {
                    let line = self.source.read_utf8_line_strict()?;
                    id = if line.is_empty() { None } else { Some(line) };
                }
                10..=12 => {
                    id = None; // 'id' on a line of its own.
                }
                13..=14 => {
                    let line = self.source.read_utf8_line_strict()?;
                    r#type = if line.is_empty() { None } else { Some(line) };
                }
                15..=17 => {
                    r#type = None; // 'event' on a line of its own
                }
                18..=19 => {
                    let retry_ms = self.read_retry_ms()?;
                    if retry_ms != -1 {
                        self.callback.on_retry_change(retry_ms);
                    }
                }
                -1 => {
                    let line_end = self.source.index_of_element(&CRLF);
                    if line_end != -1 {
                        // Skip the line and newline
                        self.source.skip(line_end);
                        // The original Kotlin code calls select(options) again here.
                        // In the loop, we just continue to the next iteration which calls select.
                        continue;
                    } else {
                        return Ok(false); // No more newlines.
                    }
                }
                _ => {
                    panic!("AssertionError");
                }
            }
        }
    }

    fn complete_event(
        &mut self,
        id: Option<String>,
        r#type: Option<String>,
        mut data: Buffer,
    ) -> Result<(), IOException> {
        if data.size() != 0 {
            self.last_id = id.clone();
            data.skip(1); // Leading newline.
            self.callback.on_event(id, r#type, data.read_utf8());
        }
        Ok(())
    }

    fn read_data(&mut self, data: &mut Buffer) -> Result<(), IOException> {
        data.write_byte(b'\n');
        let index = self.source.index_of_element(&CRLF);
        self.source.read_fully(data, index);
        self.source.select(&OPTIONS); // Skip the newline bytes.
        Ok(())
    }

    fn read_retry_ms(&mut self) -> Result<i64, IOException> {
        let retry_string = self.source.read_utf8_line_strict()?;
        Ok(retry_string.parse::<i64>().unwrap_or(-1))
    }
}

lazy_static::lazy_static! {
    static ref CRLF: ByteString = ByteString::encode_utf8("\r\n");
    static ref OPTIONS: Options = Options::of(vec![
        ByteString::encode_utf8("\r\n"),       // 0
        ByteString::encode_utf8("\r"),         // 1
        ByteString::encode_utf8("\n"),         // 2
        ByteString::encode_utf8("data: "),     // 3
        ByteString::encode_utf8("data:"),      // 4
        ByteString::encode_utf8("data\r\n"),   // 5
        ByteString::encode_utf8("data\r"),     // 6
        ByteString::encode_utf8("data\n"),     // 7
        ByteString::encode_utf8("id: "),       // 8
        ByteString::encode_utf8("id:"),        // 9
        ByteString::encode_utf8("id\r\n"),     // 10
        ByteString::encode_utf8("id\r"),       // 11
        ByteString::encode_utf8("id\n"),       // 12
        ByteString::encode_utf8("event: "),    // 13
        ByteString::encode_utf8("event:"),     // 14
        ByteString::encode_utf8("event\r\n"),  // 15
        ByteString::encode_utf8("event\r"),    // 16
        ByteString::encode_utf8("event\n"),    // 17
        ByteString::encode_utf8("retry: "),    // 18
        ByteString::encode_utf8("retry:"),     // 19
    ]);
}
