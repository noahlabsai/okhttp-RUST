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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::HandshakeCertificates::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::SingleAndroidTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// Mocking Android Build.VERSION.SDK_INT as it's a platform-specific constant
pub struct BuildVersion;
impl BuildVersion {
    pub const SDK_INT: i32 = 23; // This value would typically be provided by the environment
}

// Mocking X509Certificate and decode_certificate_pem as they are Java/TLS specific
#[derive(Debug, Clone, PartialEq)]
pub struct X509Certificate(pub Vec<u8>);

pub trait CertificatePemExt {
    fn decode_certificate_pem(self) -> X509Certificate;
}

impl CertificatePemExt for String {
    fn decode_certificate_pem(self) -> X509Certificate {
        // In a real implementation, this would parse the PEM string into a certificate
        X509Certificate(self.into_bytes())
    }
}

pub struct LetsEncryptClientTest;

impl LetsEncryptClientTest {
    #[test]
    pub async fn get() -> Result<(), Box<dyn std::error::Error>> {
        // These tests wont actually run before Android 8.0 as per
        // https://github.com/mannodermaus/android-junit5
        // Raised https://github.com/mannodermaus/android-junit5/issues/228 to reevaluate
        let android_or_earlier = BuildVersion::SDK_INT <= 23;

        let mut client_builder = OkHttpClientBuilder::new();

        if android_or_earlier {
            let cert_pem = r#"
-----BEGIN CERTIFICATE-----
MIIFazCCA1OgAwIBAgIRAIIQz7DSQONZRGPgu2OCiwAwDQYJKoZIhvcNAQELBQAw
TzELMAkGA1UEBhMCVVMxKTAnBgNVBAoTIEludGVybmV0IFNlY3VyaXR5IFJlc2Vh
cmNoIEdyb3VwMRUwEwYDVQQDEwxJU1JHIFJvb3QgWDEwHhcNMTUwNjA0MTEwNDM4
WhcNMzUwNjA0MTEwNDM4WjBPMQswCQYDVQQGEwJVUzEpMCcGA1UEChMgSW50ZXJu
ZXQgU2VjdXJpdHkgUmVzZWFyY2ggR3JvdXAxFTATBgNVBAMTDElTUkcgUm9vdCBY
MTCCAiIwDQYJKoZIhvcNAQEBBQADggIPADCCAgoCggIBAK3oJHP0FDfzm54rVygc
h77ct984kIxuPOZXoHj3dcKi/vVqbvYATyjb3miGbESTtrFj/RQSa78f0uoxmyF+
0TM8ukj13Xnfs7j/EvEhmkvBioZxaUpmZmyPfjxwv60pIgbz5MDmgK7iS4+3mX6U
A5/TR5d8mUgjU+g4rk8Kb4Mu0UlXjIB0ttov0DiNewNwIRt18jA8+o+u3dpjq+sW
T8KOEUt+zwvo/7V3LvSye0rgTBIlDHCNAymg4VMk7BPZ7hm/ELNKjD+Jo2FR3qyH
B5T0Y3HsLuJvW5iB4YlcNHlsdu87kGJ55tukmi8mxdAQ4Q7e2RCOFvu396j3x+UC
B5iPNgiV5+I3lg02dZ77DnKxHZu8A/lJBdiB3QW0KtZB6awBdpUKD9jf1b0SHzUv
KBds0pjBqAlkd25HN7rOrFleaJ1/ctaJxQZBKT5ZPt0m9STJEadao0xAH0ahmbWn
OlFuhjuefXKnEgV4We0+UXgVCwOPjdAvBbI+e0ocS3MFEvzG6uBQE3xDk3SzynTn
jh8BCNAw1FtxNrQHusEwMFxIt4I7mKZ9YIqioymCzLq9gwQbooMDQaHWBfEbwrbw
qHyGO0aoSCqI3Haadr8faqU9GY/rOPNk3sgrDQoo//fb4hVC1CLQJ13hef4Y53CI
rU7m2Ys6xt0nUW7/vGT1M0NPAgMBAAGjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNV
HRMBAf8EBTADAQH/MB0GA1UdDgQWBBR5tFnme7bl5AFzgAiIyBpY9umbbjANBgkq
hkiG9w0BAQsFAAOCAgEAVR9YqbyyqFDQDLHYGmkgJykIrGF1XIpu+ILlaS/V9lZL
ubhzEFnTIZd+50xx+7LSYK05qAvqFyFWhfFQDlnrzuBZ6brJFe+GnY+EgPbk6ZGQ
3BebYhtF8GaV0nxvwuo77x/Py9auJ/GpsMiu/X1+mvoiBOv/2X/qkSsisRcOj/KK
NFtY2PwByVS5uCbMiogziUwthDyC3+6WVwW6LLv3xLfHTjuCvjHIInNzktHCgKQ5
ORAzI4JMPJ+GslWYHb4phowim57iaztXOoJwTdwJx4nLCgdNbOhdjsnvzqvHu7Ur
TkXWStAmzOVyyghqpZXjFaH3pO3JLF+l+/+sKAIuvtd7u+Nxe5AW0wdeRlN8NwdC
jNPElpzVmbUq4JUagEiuTDkHzsxHpFKVK7q4+63SM1N95R1NbdWhscdCb+ZAJzVc
oyi3B43njTOQ5yOf+1CceWxG1bQVs5ZufpsMljq4Ui0/1lvh+wjChP4kqKOJ2qxq
4RgqsahDYVvTH9w7jXbyLeiNdd8XM2w9U/t7y0Ff/9yi0GE44Za4rF2LN9d11TPA
mRGunUHBcnWEvgJBQl9nJEiU0Zsnvgc/ubhPgXRR4Xq37Z0j4r7g1SgEEzwxA57d
emyPxgcYxn/eR44/KJ4EBs+lVDR3veyJm+kXQ99b21/+jh5Xos1AnX5iItreGCc=
-----END CERTIFICATE-----
"#;
            let cert = cert_pem.trim().to_string().decode_certificate_pem();

            let handshake_certificates = HandshakeCertificatesBuilder::new()
                // .add_platform_trusted_certificates() // TODO reenable in official answers
                .add_trusted_certificate(cert)
                .build();

            client_builder.ssl_socket_factory(
                handshake_certificates.ssl_socket_factory(),
                handshake_certificates.trust_manager,
            );
        }

        let client = client_builder.build();

        let request = RequestBuilder::new()
            .url("https://valid-isrgrootx1.letsencrypt.org/robots.txt")
            .build();

        let response = client.new_call(request).execute()?;
        
        // Using a scope or explicit drop to mimic Kotlin's .r#use { }
        {
            assert_eq!(response.code, 404);
            assert_eq!(response.protocol, Protocol::HTTP_2);
        }

        Ok(())
    }
}