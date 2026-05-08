use std::collections::HashMap;
use num_bigint::BigInt;
use std::error::Error;

// Assuming these types are defined in the same crate/module as per the project structure
// If they were in different modules, they would be imported.
#[derive(Debug, Clone, PartialEq)]
pub struct ByteString(pub Vec<u8>);


// Mocking ObjectIdentifiers as it's a dependency not provided in the source but used
pub struct ObjectIdentifiers;
impl ObjectIdentifiers {
    pub const COMMON_NAME: &'static str = "2.5.4.3";
    pub const ORGANIZATIONAL_UNIT_NAME: &'static str = "2.5.4.11";
    pub const SUBJECT_ALTERNATIVE_NAME: &'static str = "2.5.29.17";
    pub const BASIC_CONSTRAINTS: &'static str = "2.5.29.19";
    pub const SHA256_WITH_RSA_ENCRYPTION: &'static str = "1.2.840.113549.1.1.11";
    pub const SHA256_WITH_ECDSA: &'static str = "1.2.840.10045.4.3.2";
}

// Mocking CertificateAdapters as it's a dependency used for DER encoding
pub struct CertificateAdapters;
impl CertificateAdapters {
    pub struct TbsCertificateAdapter;
    pub struct CertificateAdapter;

    pub fn tbs_certificate() -> Self::TbsCertificateAdapter { Self::TbsCertificateAdapter }
    pub fn certificate() -> Self::CertificateAdapter { Self::CertificateAdapter }
}

impl CertificateAdapters::TbsCertificateAdapter {
    pub fn to_der(&self, _tbs: &TbsCertificate) -> Vec<u8> {
        // Implementation would be provided by the actual DER encoder
        Vec::new()
    }
}

impl CertificateAdapters::CertificateAdapter {
    pub fn to_der(&self, _cert: &Certificate) -> Vec<u8> {
        // Implementation would be provided by the actual DER encoder
        Vec::new()
    }
}

// Mocking Java Security types as they are platform-specific
pub struct PublicKey;
pub struct X509Certificate;
pub struct Signature;
impl Signature {
    pub fn get_instance(_algorithm: &str) -> Self { Signature }
    pub fn init_verify(&mut self, _key: PublicKey) {}
    pub fn update(&mut self, _data: &[u8]) {}
    pub fn verify(&self, _signature: &[u8]) -> bool { true }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Certificate {
    pub tbs_certificate: TbsCertificate,
    pub signature_algorithm: AlgorithmIdentifier,
    pub signature_value: BitString,
}

impl Certificate {
    pub fn common_name(&self) -> Option<&dyn std::any::Any> {
        self.tbs_certificate.subject
            .iter()
            .flatten()
            .find(|it| it.r#type == ObjectIdentifiers::COMMON_NAME)
            .and_then(|it| it.value.as_ref())
            .map(|v| v.as_ref())
    }

    pub fn organizational_unit_name(&self) -> Option<&dyn std::any::Any> {
        self.tbs_certificate.subject
            .iter()
            .flatten()
            .find(|it| it.r#type == ObjectIdentifiers::ORGANIZATIONAL_UNIT_NAME)
            .and_then(|it| it.value.as_ref())
            .map(|v| v.as_ref())
    }

    pub fn subject_alternative_names(&self) -> Option<&Extension> {
        self.tbs_certificate.extensions.iter().find(|it| {
            it.id == ObjectIdentifiers::SUBJECT_ALTERNATIVE_NAME
        })
    }

    pub fn basic_constraints(&self) -> &Extension {
        self.tbs_certificate.extensions.iter().find(|it| {
            it.id == ObjectIdentifiers::BASIC_CONSTRAINTS
        }).expect("Basic Constraints extension missing")
    }

    pub fn check_signature(&self, issuer: PublicKey) -> Result<bool, Box<dyn Error>> {
        let signed_data = CertificateAdapters::tbs_certificate().to_der(&self.tbs_certificate);
        
        let mut sig = Signature::get_instance(&self.tbs_certificate.signature_algorithm_name());
        sig.init_verify(issuer);
        sig.update(&signed_data);
        Ok(sig.verify(&self.signature_value.byte_string.0))
    }

    pub fn to_x509_certificate(&self) -> Result<X509Certificate, Box<dyn Error>> {
        let _data = CertificateAdapters::certificate().to_der(self);
        // In a real Rust implementation, this would use a crate like `x509-parser` or `openssl`
        // Since the original code calls Java's CertificateFactory, we simulate the error handling.
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "failed to decode certificate")))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TbsCertificate {
    pub version: i64,
    pub serial_number: BigInt,
    pub signature: AlgorithmIdentifier,
    pub issuer: Vec<Vec<AttributeTypeAndValue>>,
    pub validity: Validity,
    pub subject: Vec<Vec<AttributeTypeAndValue>>,
    pub subject_public_key_info: SubjectPublicKeyInfo,
    pub issuer_unique_id: Option<BitString>,
    pub subject_unique_id: Option<BitString>,
    pub extensions: Vec<Extension>,
}

impl TbsCertificate {
    pub fn signature_algorithm_name(&self) -> String {
        match self.signature.algorithm.as_str() {
            ObjectIdentifiers::SHA256_WITH_RSA_ENCRYPTION => "SHA256WithRSA".to_string(),
            ObjectIdentifiers::SHA256_WITH_ECDSA => "SHA256withECDSA".to_string(),
            _ => panic!("unexpected signature algorithm: {}", self.signature.algorithm),
        }
    }
}

// Custom Hash implementation to match Kotlin's logic (avoiding Long.hashCode issues)
impl std::hash::Hash for TbsCertificate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
use crate::okhttp_bom::build_gradle::*;
use crate::okhttp_testing_support::build_gradle::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::BitString::*;

        (self.version as i32).hash(state);
        self.serial_number.hash(state);
        self.signature.hash(state);
        self.issuer.hash(state);
        self.validity.hash(state);
        self.subject.hash(state);
        self.subject_public_key_info.hash(state);
        self.issuer_unique_id.hash(state);
        self.subject_unique_id.hash(state);
        self.extensions.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, std::hash::Hash)]
pub struct AlgorithmIdentifier {
    pub algorithm: String,
    pub parameters: Option<Box<dyn std::any::Any>>,
}

// Since Box<dyn Any> doesn't implement Hash/Eq, we use a wrapper or custom impl
// For the sake of this translation, we'll implement a basic version.
impl AlgorithmIdentifier {
    pub fn new(algorithm: String, parameters: Option<Box<dyn std::any::Any>>) -> Self {
        Self { algorithm, parameters }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeTypeAndValue {
    pub r#type: String,
    pub value: Option<Box<dyn std::any::Any>>,
}

impl std::hash::Hash for AttributeTypeAndValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        // value is Any, which cannot be hashed easily. 
        // In production, this would be a sealed trait or enum.
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Validity {
    pub not_before: i64,
    pub not_after: i64,
}

impl std::hash::Hash for Validity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.not_before as i32).hash(state);
        (self.not_after as i32).hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, std::hash::Hash)]
pub struct SubjectPublicKeyInfo {
    pub algorithm: AlgorithmIdentifier,
    pub subject_public_key: BitString,
}

impl std::hash::Hash for BitString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.byte_string.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    pub id: String,
    pub critical: bool,
    pub value: Option<Box<dyn std::any::Any>>,
}

impl std::hash::Hash for Extension {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.critical.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicConstraints {
    pub ca: bool,
    pub max_intermediate_cas: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateKeyInfo {
    pub version: i64,
    pub algorithm_identifier: AlgorithmIdentifier,
    pub private_key: ByteString,
}

impl std::hash::Hash for PrivateKeyInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.version as i32).hash(state);
        self.algorithm_identifier.hash(state);
        self.private_key.0.hash(state);
    }
}