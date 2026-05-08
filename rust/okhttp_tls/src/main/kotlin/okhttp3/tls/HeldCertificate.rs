use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::AlgorithmIdentifier;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::AttributeTypeAndValue;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::BasicConstraints;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::BitString;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::Certificate;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::CertificateAdapters;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::Extension;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::ObjectIdentifiers;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::TbsCertificate;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::Validity;
use num_bigint::BigInt;
use okio::ByteString;
use regex::Regex;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::BitString::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::CertificateAdapters::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::ObjectIdentifiers::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::letsencrypt::LetsEncryptClientTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// Mocking Java Security types as they are platform-specific. 
// In a real production system, these would be wrappers around a crate like `ring` or `openssl`.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyPair {
    pub public: PublicKey,
    pub private: PrivateKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey {
    pub encoded: Vec<u8>,
    pub key_type: String, // "RSA" or "EC"
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateKey {
    pub encoded: Vec<u8>,
    pub key_type: String, // "RSA" or "EC"
}


impl X509Certificate {
    pub fn certificate_pem(&self) -> String {
        format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            base64::encode(&self.encoded)
        )
    }
}

pub struct HeldCertificate {
    pub key_pair: KeyPair,
    pub certificate: X509Certificate,
}

impl HeldCertificate {
    pub fn new(key_pair: KeyPair, certificate: X509Certificate) -> Self {
        Self { key_pair, certificate }
    }

    pub fn certificate_pem(&self) -> String {
        self.certificate.certificate_pem()
    }

    pub fn private_key_pkcs8_pem(&self) -> String {
        let mut result = String::from("-----BEGIN PRIVATE KEY-----\n");
        let bytes = ByteString::from(self.key_pair.private.encoded.clone());
        result.push_str(&encode_base64_lines(bytes));
        result.push_str("-----END PRIVATE KEY-----\n");
        result
    }

    pub fn private_key_pkcs1_pem(&self) -> Result<String, Box<dyn std::error::Error>> {
        if self.key_pair.private.key_type != "RSA" {
            return Err("PKCS1 only supports RSA keys".into());
        }
        let mut result = String::from("-----BEGIN RSA PRIVATE KEY-----\n");
        let bytes = self.pkcs1_bytes()?;
        result.push_str(&encode_base64_lines(bytes));
        result.push_str("-----END RSA PRIVATE KEY-----\n");
        Ok(result)
    }

    fn pkcs1_bytes(&self) -> Result<ByteString, Box<dyn std::error::Error>> {
        let encoded = ByteString::from(self.key_pair.private.encoded.clone());
        let decoded = CertificateAdapters::private_key_info().from_der(&mut encoded);
        Ok(decoded.private_key)
    }

    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn decode(certificate_and_private_key_pem: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pem_regex = Regex::new(r#"-----BEGIN ([!-,.-~ ]*)-----([^-]*)-----END \1-----"#).unwrap();
        let mut certificate_pem: Option<String> = None;
        let mut pkcs8_base64: Option<String> = None;

        for cap in pem_regex.captures_iter(certificate_and_private_key_pem) {
            let label = &cap[1];
            match label {
                "CERTIFICATE" => {
                    if certificate_pem.is_some() {
                        return Err("string includes multiple certificates".into());
                    }
                    certificate_pem = Some(cap[0].to_string());
                }
                "PRIVATE KEY" => {
                    if pkcs8_base64.is_some() {
                        return Err("string includes multiple private keys".into());
                    }
                    pkcs8_base64 = Some(cap[2].to_string());
                }
                _ => return Err(format!("unexpected type: {}", label).into()),
            }
        }

        let cert_pem = certificate_pem.ok_or("string does not include a certificate")?;
        let pkcs8_text = pkcs8_base64.ok_or("string does not include a private key")?;

        Self::decode_internal(cert_pem, pkcs8_text)
    }

    fn decode_internal(certificate_pem: String, pkcs8_base64_text: String) -> Result<Self, Box<dyn std::error::Error>> {
        let certificate = decode_certificate_pem(certificate_pem)?;
        
        let pkcs8_bytes = base64::decode(pkcs8_base64_text)
            .map(|b| ByteString::from(b))
            .map_err(|_| "failed to decode private key")?;

        let key_type = match certificate.public_key.key_type.as_str() {
            "EC" => "EC",
            "RSA" => "RSA",
            _ => return Err(format!("unexpected key type: {}", certificate.public_key.key_type).into()),
        };

        let private_key = decode_pkcs8(pkcs8_bytes, key_type)?;
        let key_pair = KeyPair {
            public: certificate.public_key.clone(),
            private: private_key,
        };

        Ok(HeldCertificate { key_pair, certificate })
    }
}


impl Default for Builder {
    fn default() -> Self {
        let mut b = Self {
            not_before: -1,
            not_after: -1,
            common_name: None,
            organizational_unit: None,
            alt_names: Vec::new(),
            serial_number: None,
            key_pair: None,
            signed_by: None,
            max_intermediate_cas: -1,
            key_algorithm: None,
            key_size: 0,
        };
        b.ecdsa256();
        b
    }
}

impl Builder {
    const DEFAULT_DURATION_MILLIS: i64 = 1000 * 60 * 60 * 24;

    pub fn validity_interval(mut self, not_before: i64, not_after: i64) -> Self {
        if !(not_before <= not_after && (not_before == -1) == (not_after == -1)) {
            panic!("invalid interval: {}..{}", not_before, not_after);
        }
        self.not_before = not_before;
        self.not_after = not_after;
        self
    }

    pub fn duration(self, duration: i64, unit_millis: i64) -> Self {
        let now = current_time_millis();
        self.validity_interval(now, now + duration * unit_millis)
    }

    pub fn add_subject_alternative_name(mut self, alt_name: String) -> Self {
        self.alt_names.push(alt_name);
        self
    }

    pub fn common_name(mut self, cn: String) -> Self {
        self.common_name = Some(cn);
        self
    }

    pub fn organizational_unit(mut self, ou: String) -> Self {
        self.organizational_unit = Some(ou);
        self
    }

    pub fn serial_number_bigint(mut self, sn: BigInt) -> Self {
        self.serial_number = Some(sn);
        self
    }

    pub fn serial_number_long(self, sn: i64) -> Self {
        self.serial_number_bigint(BigInt::from(sn))
    }

    pub fn key_pair(mut self, key_pair: KeyPair) -> Self {
        self.key_pair = Some(key_pair);
        self
    }

    pub fn signed_by(mut self, signed_by: Option<HeldCertificate>) -> Self {
        self.signed_by = signed_by.map(Box::new);
        self
    }

    pub fn certificate_authority(mut self, max_intermediate_cas: i32) -> Self {
        if max_intermediate_cas < 0 {
            panic!("maxIntermediateCas < 0: {}", max_intermediate_cas);
        }
        self.max_intermediate_cas = max_intermediate_cas;
        self
    }

    pub fn ecdsa256(mut self) -> Self {
        self.key_algorithm = Some("EC".to_string());
        self.key_size = 256;
        self
    }

    pub fn rsa2048(mut self) -> Self {
        self.key_algorithm = Some("RSA".to_string());
        self.key_size = 2048;
        self
    }

    pub fn build(self) -> Result<HeldCertificate, Box<dyn std::error::Error>> {
        let subject_key_pair = self.key_pair.unwrap_or_else(|| self.generate_key_pair());
        
        let mut subject_pub_encoded = ByteString::from(subject_key_pair.public.encoded.clone());
        let subject_public_key_info = CertificateAdapters::subject_public_key_info().from_der(&mut subject_pub_encoded);
        
        let subject = self.subject();

        let (issuer_key_pair, issuer) = if let Some(ref signed_by) = self.signed_by {
            let issuer_key_pair = signed_by.key_pair.clone();
            let mut issuer_encoded = ByteString::from(signed_by.certificate.subject_x500_principal_encoded.clone());
            let issuer = CertificateAdapters::rdn_sequence().from_der(&mut issuer_encoded);
            (issuer_key_pair, issuer)
        } else {
            (subject_key_pair.clone(), subject.clone())
        };

        let signature_algorithm = self.signature_algorithm(&issuer_key_pair);

        let tbs_certificate = TbsCertificate {
            version: 2,
            serial_number: self.serial_number.clone().unwrap_or_else(|| BigInt::from(1)),
            signature: signature_algorithm.clone(),
            issuer,
            validity: self.validity(),
            subject,
            subject_public_key_info,
            issuer_unique_id: None,
            subject_unique_id: None,
            extensions: self.extensions(),
        };

        let signature_bytes = mock_sign(
            &tbs_certificate.signature_algorithm_name(),
            &issuer_key_pair.private,
            &CertificateAdapters::tbs_certificate().to_der(&tbs_certificate)
        );

        let certificate = Certificate {
            tbs_certificate,
            signature_algorithm,
            signature_value: BitString::new(ByteString::from(signature_bytes), 0),
        };

        Ok(HeldCertificate::new(subject_key_pair, certificate.to_x509_certificate()?))
    }

    fn subject(&self) -> Vec<Vec<AttributeTypeAndValue>> {
        let mut result = Vec::new();
        if let Some(ref ou) = self.organizational_unit {
            result.push(vec![AttributeTypeAndValue {
                r#type: ObjectIdentifiers::ORGANIZATIONAL_UNIT_NAME.to_string(),
                value: Some(Box::new(ou.clone())),
            }]);
        }
        result.push(vec![AttributeTypeAndValue {
            r#type: ObjectIdentifiers::COMMON_NAME.to_string(),
            value: Some(Box::new(self.common_name.clone().unwrap_or_else(|| Uuid::new_v4().to_string()))),
        }]);
        result
    }

    fn validity(&self) -> Validity {
        let not_before = if self.not_before != -1 { self.not_before } else { current_time_millis() };
        let not_after = if self.not_after != -1 { self.not_after } else { not_before + Self::DEFAULT_DURATION_MILLIS };
        Validity { not_before, not_after }
    }

    fn extensions(&self) -> Vec<Extension> {
        let mut result = Vec::new();
        if self.max_intermediate_cas != -1 {
            result.push(Extension {
                id: ObjectIdentifiers::BASIC_CONSTRAINTS.to_string(),
                critical: true,
                value: Some(Box::new(BasicConstraints {
                    ca: true,
                    max_intermediate_cas: Some(self.max_intermediate_cas as i64),
                })),
            });
        }
        if !self.alt_names.is_empty() {
            let extension_value: Vec<(String, ByteString)> = self.alt_names.iter().map(|it| {
                if let Ok(ip) = it.parse::<IpAddr>() {
                    (
                        "generalNameIpAddress".to_string(), 
                        ByteString::from(ip.octets().to_vec())
                    )
                } else {
                    (
                        "generalNameDnsName".to_string(), 
                        ByteString::from(it.as_bytes())
                    )
                }
            }).collect();
            
            result.push(Extension {
                id: ObjectIdentifiers::SUBJECT_ALTERNATIVE_NAME.to_string(),
                critical: true,
                value: Some(Box::new(extension_value)),
            });
        }
        result
    }

    fn signature_algorithm(&self, signed_by_key_pair: &KeyPair) -> AlgorithmIdentifier {
        if signed_by_key_pair.private.key_type == "RSA" {
            AlgorithmIdentifier::new(
                ObjectIdentifiers::SHA256_WITH_RSA_ENCRYPTION.to_string(),
                None,
            )
        } else {
            AlgorithmIdentifier::new(
                ObjectIdentifiers::SHA256_WITH_ECDSA.to_string(),
                Some(Box::new(ByteString::from(vec![]))),
            )
        }
    }

    fn generate_key_pair(&self) -> KeyPair {
        KeyPair {
            public: PublicKey { encoded: vec![], key_type: self.key_algorithm.clone().unwrap_or_else(|| "EC".to_string()) },
            private: PrivateKey { encoded: vec![], key_type: self.key_algorithm.clone().unwrap_or_else(|| "EC".to_string()) },
        }
    }
}

fn current_time_millis() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

fn encode_base64_lines(bytes: ByteString) -> String {
    let encoded = base64::encode(bytes.as_ref());
    encoded.as_bytes()
        .chunks(64)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_certificate_pem(pem: String) -> Result<X509Certificate, Box<dyn std::error::Error>> {
    Ok(X509Certificate {
        encoded: vec![],
        public_key: PublicKey { encoded: vec![], key_type: "RSA".to_string() },
        subject_x500_principal_encoded: vec![],
    })
}

fn decode_pkcs8(data: ByteString, key_algorithm: &str) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    Ok(PrivateKey {
        encoded: data.as_ref().to_vec(),
        key_type: key_algorithm.to_string(),
    })
}

fn mock_sign(_alg: &str, _key: &PrivateKey, _data: &[u8]) -> Vec<u8> {
    vec![0u8; 64]
}
