use std::collections::HashSet;
use okio::ByteString;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::HeldCertificate;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CertificatePinner;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CertificatePinner::Pin;

/// Mocking the SSLPeerUnverifiedException as it's a specific Java exception.
#[derive(Debug, Clone, PartialEq)]
pub struct SslPeerUnverifiedException(pub String);
impl std::fmt::Display for SslPeerUnverifiedException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for SslPeerUnverifiedException {}

pub struct CertificatePinnerTest;

impl CertificatePinnerTest {
    #[test]
    pub fn malformed_pin() {
        let builder = CertificatePinner::Builder::default();
        let result = std::panic::catch_unwind(|| {
            builder.add("example.com", "md5/DmxUShsZuNiqPQsX2Oi9uv2sCnw=");
        });
        assert!(result.is_err(), "Expected IllegalArgumentException for malformed pin");
    }

    #[test]
    pub fn malformed_base64() {
        let builder = CertificatePinner::Builder::default();
        let result = std::panic::catch_unwind(|| {
            builder.add("example.com", "sha1/DmxUShsZuNiqPQsX2Oi9uv2sCnw*");
        });
        assert!(result.is_err(), "Expected IllegalArgumentException for malformed base64");
    }

    #[test]
    pub fn same_keypair_same_pin() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        let cert_b1_sha256_pin = Self::cert_b1_sha256_pin();

        let held_certificate_a2 = HeldCertificate::Builder::default()
            .key_pair(cert_a1.key_pair.clone())
            .serial_number_long(101)
            .build()
            .expect("Failed to build cert A2");
        
        let keypair_a_certificate_2_pin = CertificatePinner::pin(&held_certificate_a2.certificate);
        
        let held_certificate_b2 = HeldCertificate::Builder::default()
            .key_pair(cert_b1.key_pair.clone())
            .serial_number_long(201)
            .build()
            .expect("Failed to build cert B2");
            
        let keypair_b_certificate_2_pin = CertificatePinner::pin(&held_certificate_b2.certificate);

        assert_eq!(keypair_a_certificate_2_pin, cert_a1_sha256_pin);
        assert_eq!(keypair_b_certificate_2_pin, cert_b1_sha256_pin);
        assert_ne!(cert_b1_sha256_pin, cert_a1_sha256_pin);
    }

    #[test]
    pub fn successful_check() {
        let cert_a1 = Self::cert_a1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        
        let certificate_pinner = CertificatePinner::Builder::default()
            .add("example.com", &cert_a1_sha256_pin)
            .build();
            
        certificate_pinner.check("example.com", vec![cert_a1.certificate]).expect("Check should succeed");
    }

    #[test]
    pub fn successful_match_accepts_any_matching_certificate() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_b1_sha256_pin = Self::cert_b1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("example.com", &cert_b1_sha256_pin)
            .build();
            
        certificate_pinner.check("example.com", vec![cert_a1.certificate, cert_b1.certificate]).expect("Check should succeed");
    }

    #[test]
    pub fn unsuccessful_check() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("example.com", &cert_a1_sha256_pin)
            .build();
            
        let result = certificate_pinner.check("example.com", vec![cert_b1.certificate]);
        assert!(result.is_err(), "Expected SSLPeerUnverifiedException");
    }

    #[test]
    pub fn multiple_certificates_for_one_hostname() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        let cert_b1_sha256_pin = Self::cert_b1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("example.com", &cert_a1_sha256_pin, &cert_b1_sha256_pin)
            .build();
            
        certificate_pinner.check("example.com", vec![cert_a1.certificate]).expect("Check A should succeed");
        certificate_pinner.check("example.com", vec![cert_b1.certificate]).expect("Check B should succeed");
    }

    #[test]
    pub fn multiple_hostnames_for_one_certificate() {
        let cert_a1 = Self::cert_a1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("example.com", &cert_a1_sha256_pin)
            .add("www.example.com", &cert_a1_sha256_pin)
            .build();
            
        certificate_pinner.check("example.com", vec![cert_a1.certificate.clone()]).expect("Check 1 should succeed");
        certificate_pinner.check("www.example.com", vec![cert_a1.certificate]).expect("Check 2 should succeed");
    }

    #[test]
    pub fn absent_hostname_matches() {
        let cert_a1 = Self::cert_a1();
        let certificate_pinner = CertificatePinner::Builder::default().build();
        certificate_pinner.check("example.com", vec![cert_a1.certificate]).expect("Absent hostname should match");
    }

    #[test]
    pub fn successful_check_for_wildcard_hostname() {
        let cert_a1 = Self::cert_a1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("*.example.com", &cert_a1_sha256_pin)
            .build();
            
        certificate_pinner.check("a.example.com", vec![cert_a1.certificate]).expect("Wildcard check should succeed");
    }

    #[test]
    pub fn successful_match_accepts_any_matching_certificate_for_wildcard_hostname() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_b1_sha256_pin = Self::cert_b1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("*.example.com", &cert_b1_sha256_pin)
            .build();
            
        certificate_pinner.check("a.example.com", vec![cert_a1.certificate, cert_b1.certificate]).expect("Wildcard match should succeed");
    }

    #[test]
    pub fn unsuccessful_check_for_wildcard_hostname() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("*.example.com", &cert_a1_sha256_pin)
            .build();
            
        let result = certificate_pinner.check("a.example.com", vec![cert_b1.certificate]);
        assert!(result.is_err(), "Expected SSLPeerUnverifiedException for wildcard");
    }

    #[test]
    pub fn multiple_certificates_for_one_wildcard_hostname() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        let cert_b1_sha256_pin = Self::cert_b1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("*.example.com", &cert_a1_sha256_pin, &cert_b1_sha256_pin)
            .build();
            
        certificate_pinner.check("a.example.com", vec![cert_a1.certificate]).expect("Wildcard A should succeed");
        certificate_pinner.check("a.example.com", vec![cert_b1.certificate]).expect("Wildcard B should succeed");
    }

    #[test]
    pub fn successful_check_for_one_hostname_with_wildcard_and_direct_certificate() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        let cert_b1_sha256_pin = Self::cert_b1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("*.example.com", &cert_a1_sha256_pin)
            .add("a.example.com", &cert_b1_sha256_pin)
            .build();
            
        certificate_pinner.check("a.example.com", vec![cert_a1.certificate]).expect("Wildcard match should succeed");
        certificate_pinner.check("a.example.com", vec![cert_b1.certificate]).expect("Direct match should succeed");
    }

    #[test]
    pub fn unsuccessful_check_for_one_hostname_with_wildcard_and_direct_certificate() {
        let cert_c1 = Self::cert_c1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        let cert_b1_sha256_pin = Self::cert_b1_sha256_pin();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("*.example.com", &cert_a1_sha256_pin)
            .add("a.example.com", &cert_b1_sha256_pin)
            .build();
            
        let result = certificate_pinner.check("a.example.com", vec![cert_c1.certificate]);
        assert!(result.is_err(), "Expected SSLPeerUnverifiedException");
    }

    #[test]
    pub fn check_for_hostname_with_double_asterisk() {
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        let cert_b1 = Self::cert_b1();

        let certificate_pinner = CertificatePinner::Builder::default()
            .add("**.example.co.uk", &cert_a1_sha256_pin)
            .build();

        // Should be pinned (fail because we use cert_b1)
        assert!(certificate_pinner.check("example.co.uk", vec![cert_b1.certificate.clone()]).is_err());
        assert!(certificate_pinner.check("foo.example.co.uk", vec![cert_b1.certificate.clone()]).is_err());
        assert!(certificate_pinner.check("foo.bar.example.co.uk", vec![cert_b1.certificate.clone()]).is_err());
        assert!(certificate_pinner.check("foo.bar.baz.example.co.uk", vec![cert_b1.certificate]).is_err());

        // Should not be pinned
        certificate_pinner.check("uk", vec![cert_b1.certificate.clone()]).expect("Should not be pinned");
        certificate_pinner.check("co.uk", vec![cert_b1.certificate.clone()]).expect("Should not be pinned");
        certificate_pinner.check("anotherexample.co.uk", vec![cert_b1.certificate.clone()]).expect("Should not be pinned");
        certificate_pinner.check("foo.anotherexample.co.uk", vec![cert_b1.certificate]).expect("Should not be pinned");
    }

    #[test]
    pub fn test_bad_pin() {
        let result = std::panic::catch_unwind(|| {
            Pin::new("example.co.uk", "sha256/a");
        });
        assert!(result.is_err(), "Expected IllegalArgumentException for bad pin");
    }

    #[test]
    pub fn test_bad_algorithm() {
        let result = std::panic::catch_unwind(|| {
            Pin::new("example.co.uk", "sha512/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
        });
        assert!(result.is_err(), "Expected IllegalArgumentException for bad algorithm");
    }

    #[test]
    pub fn test_bad_host() {
        let result = std::panic::catch_unwind(|| {
            Pin::new("example.*", "sha256/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
        });
        assert!(result.is_err(), "Expected IllegalArgumentException for bad host");
    }

    #[test]
    pub fn test_good_pin() {
        let pin = Pin::new("**.example.co.uk", "sha256/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
        
        let expected_hash = ByteString::decode_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("Valid base64");
        assert_eq!(pin.hash, expected_hash);
        assert_eq!(pin.hash_algorithm, "sha256");
        assert_eq!(pin.pattern, "**.example.co.uk");
        assert!(pin.matches_hostname("www.example.co.uk"));
        assert!(pin.matches_hostname("gopher.example.co.uk"));
        assert!(!pin.matches_hostname("www.example.com"));
    }

    #[test]
    pub fn test_matches_sha256() {
        let cert_a1 = Self::cert_a1();
        let cert_b1 = Self::cert_b1();
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        
        let pin = Pin::new("example.com", &cert_a1_sha256_pin);
        assert!(pin.matches_certificate(&cert_a1.certificate));
        assert!(!pin.matches_certificate(&cert_b1.certificate));
    }

    #[test]
    pub fn test_matches_sha1() {
        let cert_c1 = Self::cert_c1();
        let cert_b1 = Self::cert_b1();
        let cert_c1_sha1_pin = Self::cert_c1_sha1_pin();
        
        let pin = Pin::new("example.com", &cert_c1_sha1_pin);
        assert!(pin.matches_certificate(&cert_c1.certificate));
        assert!(!pin.matches_certificate(&cert_b1.certificate));
    }

    #[test]
    pub fn pin_list() {
        let cert_a1_sha256_pin = Self::cert_a1_sha256_pin();
        let mut builder = CertificatePinner::Builder::default();
        builder.add("example.com", &cert_a1_sha256_pin);
        builder.add("www.example.com", &cert_a1_sha256_pin);
        
        let certificate_pinner = builder.build();
        
        let expected_pins = vec![
            Pin::new("example.com", &cert_a1_sha256_pin),
            Pin::new("www.example.com", &cert_a1_sha256_pin),
        ];
        
        assert_eq!(builder.pins, expected_pins);
        
        let pinner_pins_set: HashSet<_> = certificate_pinner.pins.iter().cloned().collect();
        let expected_set: HashSet<_> = expected_pins.into_iter().collect();
        assert_eq!(pinner_pins_set, expected_set);
    }

    // Companion object equivalents
    fn cert_a1() -> HeldCertificate {
        HeldCertificate::Builder::default().serial_number_long(100).build().unwrap()
    }
    fn cert_b1() -> HeldCertificate {
        HeldCertificate::Builder::default().serial_number_long(200).build().unwrap()
    }
    fn cert_c1() -> HeldCertificate {
        HeldCertificate::Builder::default().serial_number_long(300).build().unwrap()
    }
    fn cert_a1_sha256_pin() -> String {
        CertificatePinner::pin(&Self::cert_a1().certificate)
    }
    fn cert_b1_sha256_pin() -> String {
        CertificatePinner::pin(&Self::cert_b1().certificate)
    }
    fn cert_c1_sha1_pin() -> String {
        let cert = Self::cert_c1().certificate;
        format!("sha1/{}", cert.sha1_hash().base64())
    }
}