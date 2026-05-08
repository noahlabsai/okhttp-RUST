use okio::{Buffer, ByteString};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

// Trivial Dns Encoder/Decoder, basically ripped from Netty full implementation.
pub struct DnsRecordCodec;

impl DnsRecordCodec {
    const SERVFAIL: i32 = 2;
    const NXDOMAIN: i32 = 3;
    pub const TYPE_A: i32 = 0x0001;
    pub const TYPE_AAAA: i32 = 0x001c;
    const TYPE_PTR: i32 = 0x000c;

    pub fn encode_query(host: &str, query_type: i32) -> ByteString {
        let mut buffer = Buffer::new();
        
        // query id
        buffer.write_short(0); 
        // flags with recursion
        buffer.write_short(256); 
        // question count
        buffer.write_short(1); 
        // answerCount
        buffer.write_short(0); 
        // authorityResourceCount
        buffer.write_short(0); 
        // additional
        buffer.write_short(0); 

        let mut name_buf = Buffer::new();
        
        // Kotlin: host.split('.').dropLastWhile { it.isEmpty() }
        let labels_forward: Vec<&str> = host.split('.').collect();
        let mut end_idx = labels_forward.len();
        while end_idx > 0 && labels_forward[end_idx - 1].is_empty() {
            end_idx -= 1;
        }

        for i in 0..end_idx {
            let label = labels_forward[i];
            let utf8_bytes = label.as_bytes();
            let utf8_byte_count = utf8_bytes.len();
            
            // require(utf8ByteCount == label.length.toLong()) { "non-ascii hostname: $host" }
            if utf8_byte_count != label.chars().count() {
                panic!("non-ascii hostname: {}", host);
            }
            
            name_buf.write_byte(utf8_byte_count as u8);
            name_buf.write_utf8(label);
        }
        name_buf.write_byte(0); // end

        // nameBuf.copyTo(this, 0, nameBuf.size)
        buffer.write_all(&name_buf.snapshot());
        
        buffer.write_short(query_type as i16);
        buffer.write_short(1); // CLASS_IN
        
        buffer.read_byte_string()
    }

    pub fn decode_answers(
        hostname: &str,
        byte_string: ByteString,
    ) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
        let mut result = Vec::new();

        let mut buf = Buffer::new();
        buf.write_all(&byte_string);
        
        buf.read_short()?; // query id

        let flags = (buf.read_short()? as i32) & 0xffff;
        if (flags >> 15) == 0 {
            return Err("not a response".into());
        }

        let response_code = flags & 0xf;

        if response_code == Self::NXDOMAIN {
            return Err(format!("{}: NXDOMAIN", hostname).into());
        } else if response_code == Self::SERVFAIL {
            return Err(format!("{}: SERVFAIL", hostname).into());
        }

        let question_count = (buf.read_short()? as i32) & 0xffff;
        let answer_count = (buf.read_short()? as i32) & 0xffff;
        buf.read_short()?; // authority record count
        buf.read_short()?; // additional record count

        for _ in 0..question_count {
            Self::skip_name(&mut buf)?; // name
            buf.read_short()?; // type
            buf.read_short()?; // class
        }

        for _ in 0..answer_count {
            Self::skip_name(&mut buf)?; // name

            let record_type = (buf.read_short()? as i32) & 0xffff;
            buf.read_short()?; // class
            let _ttl = (buf.read_int()? as i64) & 0xffffffff; // ttl
            let length = (buf.read_short()? as i32) & 0xffff;

            if record_type == Self::TYPE_A || record_type == Self::TYPE_AAAA {
                let mut bytes = vec![0u8; length as usize];
                buf.read_exact(&mut bytes)?;
                
                if record_type == Self::TYPE_A {
                    if bytes.len() == 4 {
                        let mut addr = [0u8; 4];
                        addr.copy_from_slice(&bytes);
                        result.push(IpAddr::V4(Ipv4Addr::from(addr)));
                    }
                } else if record_type == Self::TYPE_AAAA {
                    if bytes.len() == 16 {
                        let mut addr = [0u8; 16];
                        addr.copy_from_slice(&bytes);
                        result.push(IpAddr::V6(Ipv6Addr::from(addr)));
                    }
                }
            } else {
                buf.skip(length as i64)?;
            }
        }

        Ok(result)
    }

    fn skip_name(source: &mut Buffer) -> Result<(), Box<dyn std::error::Error>> {
        // 0 - 63 bytes
        let length = source.read_byte()? as i8;

        if length < 0 {
            // compressed name pointer, first two bits are 1
            // drop second byte of compression offset
            source.skip(1)?;
        } else {
            let mut current_len = length as i32;
            while current_len > 0 {
                // skip each part of the domain name
                source.skip(current_len as i64)?;
                current_len = source.read_byte()? as i8 as i32;
            }
        }
        Ok(())
    }
}

// Helper trait to mimic Okio's Buffer methods
trait OkioBufferExt: Read + Write {
    fn write_short(&mut self, value: i16) {
        self.write_all(&value.to_be_bytes()).unwrap();
    }
    fn write_byte(&mut self, value: u8) {
        self.write_all(&[value]).unwrap();
    }
    fn write_utf8(&mut self, value: &str) {
        self.write_all(value.as_bytes()).unwrap();
    }
    fn read_short(&mut self) -> Result<i16, std::io::Error> {
        let mut b = [0u8; 2];
        self.read_exact(&mut b)?;
        Ok(i16::from_be_bytes(b))
    }
    fn read_byte(&mut self) -> Result<u8, std::io::Error> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }
    fn read_int(&mut self) -> Result<i32, std::io::Error> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(i32::from_be_bytes(b))
    }
    fn skip(&mut self, amount: i64) -> Result<(), std::io::Error> {
        let mut dummy = vec![0u8; amount as usize];
        self.read_exact(&mut dummy)?;
        Ok(())
    }
}

impl OkioBufferExt for Buffer {}
