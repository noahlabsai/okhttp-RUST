use okio::{Buffer, ByteString};
use std::error::Error;
use std::fmt::Write as _;

// Mocking X509Certificate as it is a Java security class.
// In a real Rust production environment, one would use the `x509-parser` or `openssl` crate.

impl X509Certificate {
    // Mock implementation of CertificateFactory.getInstance("X.509").generateCertificates(...)
    pub fn decode_pem(pem: &str) -> Result<Self, Box<dyn Error>> {
        // This is a simplified mock of the Java CertificateFactory behavior.
        // In production, this would involve parsing the PEM headers and decoding Base64.
        if !pem.contains("-----BEGIN CERTIFICATE-----") || !pem.contains("-----END CERTIFICATE-----") {
            return Err("failed to decode certificate: Invalid PEM format".into());
        }
        
        // Extract content between headers
        let start = pem.find("-----BEGIN CERTIFICATE-----").ok_or("Missing start header")? + 27;
        let end = pem.rfind("-----END CERTIFICATE-----").ok_or("Missing end header")?;
        let base64_content = pem[start..end].replace('\n', "").replace('\r', "");
        
        let decoded = base64::decode(base64_content)
            .map_err(|e| format!("failed to decode certificate: {}", e))?;
            
        Ok(X509Certificate { encoded: decoded })
    }
}

pub trait CertificatePemExt {
    fn decode_certificate_pem(&self) -> Result<X509Certificate, Box<dyn Error>>;
}

impl CertificatePemExt for String {
    // Decodes a multiline string that contains a certificate which is PEM-encoded.
    fn decode_certificate_pem(&self) -> Result<X509Certificate, Box<dyn Error>> {
        // Kotlin: Buffer().writeUtf8(this).inputStream()
        // In Rust, we can pass the string slice directly to our decoder.
        X509Certificate::decode_pem(self).map_err(|e| {
            // Preserve the Kotlin behavior of wrapping various exceptions into IllegalArgumentException
            format!("failed to decode certificate: {}", e).into()
        })
    }
}

pub trait X509CertificateExt {
    fn certificate_pem(&self) -> String;
}

impl X509CertificateExt for X509Certificate {
    // Returns the certificate encoded in PEM format.
    fn certificate_pem(&self) -> String {
        let mut builder = String::new();
        builder.push_str("-----BEGIN CERTIFICATE-----\n");
        
        // Kotlin: encodeBase64Lines(encoded.toByteString())
        let byte_string = ByteString::from(self.encoded.clone());
        encode_base64_lines(&mut builder, byte_string);
        
        builder.push_str("-----END CERTIFICATE-----\n");
        builder
    }
}

// Internal helper to encode Base64 data into lines of 64 characters.
fn encode_base64_lines(builder: &mut String, data: ByteString) {
    // Kotlin: data.base64()
    let base64 = data.base64();
    let len = base64.len();
    
    for i in (0..len).step_by(64) {
        let end = std::cmp::min(i + 64, len);
        builder.push_str(&base64[i..end]);
        builder.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::letsencrypt::LetsEncryptClientTest::*;

    #[test]
    fn test_pem_roundtrip() {
        let pem_input = "-----BEGIN CERTIFICATE-----\n\
                        MIIBYTCCAQegAwIBAgIBKjAKBggqhkjOPQQDAjApMRQwEgYDVQQLEwtlbmdpbmVl\n\
                        cmluZzERMA8GA1UEAxMIY2FzaC5hcHAwHhcNNzAwMTAxMDAwMDA1WhcNNzAwMTAx\n\
                        MDAwMDEwWjApMRQwEgYDVQQLEwtlbmdpbmVlcmluZzERMA8GA1UEAxMIY2FzaC5h\n\
                        cHAwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASda8ChkQXxGELnrV/oBnIAx3dD\n\
                        ocUOJfdz4pOJTP6dVQB9U3UBiW5uSX/MoOD0LL5zG3bVyL3Y6pDwKuYvfLNhoyAw\n\
                        HjAcBgNVHREBAf8EEjAQhwQBAQEBgghjYXNoLmFwcDAKBggqhkjOPQQDAgNIADBF\n\
                        AiAyHHg1N6YDDQiY920+cnI5XSZwEGhAtb9PYWO8bLmkcQIhAI2CfEZf3V/obmdT\n\
                        yyaoEufLKVXhrTQhRfodTeigi4RX\n\
                        -----END CERTIFICATE-----";
        
        let cert = pem_input.to_string().decode_certificate_pem().unwrap();
        let pem_output = cert.certificate_pem();
        
        assert!(pem_output.contains("-----BEGIN CERTIFICATE-----"));
        assert!(pem_output.contains("-----END CERTIFICATE-----"));
    }
}