use std::collections::HashMap;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::letsencrypt::LetsEncryptClientTest::*;

// Mocking external dependencies as they are not provided in the source but required for compilation.
// In a real production environment, these would be imported from the actual crates.
pub trait HostnameVerifier {
    fn verify(&self, host: &str, session: &SSLSession) -> bool;
}



// Mocking internal OkHttp utility functions
mod internal_utils {
    pub fn can_parse_as_ip_address(host: &str) -> bool {
        // Implementation would check if string is a valid IP
        false 
    }
    pub fn to_canonical_host(host: &str) -> String {
        // Implementation would normalize IP/Host
        host.to_string()
    }
}

pub struct OkHostnameVerifier;

impl OkHostnameVerifier {
    const ALT_DNS_NAME: i32 = 2;
    const ALT_IPA_NAME: i32 = 7;

    pub fn verify_with_cert(&self, host: &str, certificate: &X509Certificate) -> bool {
        if internal_utils::can_parse_as_ip_address(host) {
            self.verify_ip_address(host, certificate)
        } else {
            self.verify_hostname_with_cert(host, certificate)
        }
    }

    fn verify_ip_address(&self, ip_address: &str, certificate: &X509Certificate) -> bool {
        let canonical_ip_address = internal_utils::to_canonical_host(ip_address);

        self.get_subject_alt_names(certificate, Self::ALT_IPA_NAME)
            .into_iter()
            .any(|it| canonical_ip_address == internal_utils::to_canonical_host(&it))
    }

    fn verify_hostname_with_cert(&self, hostname: &str, certificate: &X509Certificate) -> bool {
        let hostname_lower = hostname.ascii_to_lowercase();
        self.get_subject_alt_names(certificate, Self::ALT_DNS_NAME)
            .into_iter()
            .any(|it| self.verify_hostname_pattern(Some(&hostname_lower), Some(&it)))
    }

    fn verify_hostname_pattern(&self, hostname: Option<&str>, pattern: Option<&str>) -> bool {
        let mut hostname = match hostname {
            Some(h) if !h.is_empty() => h.to_string(),
            _ => return false,
        };
        let mut pattern = match pattern {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => return false,
        };

        if hostname.starts_with('.') || hostname.ends_with("..") {
            return false;
        }
        if pattern.starts_with('.') || pattern.ends_with("..") {
            return false;
        }

        if !hostname.ends_with('.') {
            hostname.push('.');
        }
        if !pattern.ends_with('.') {
            pattern.push('.');
        }

        let pattern = pattern.ascii_to_lowercase();

        if !pattern.contains('*') {
            return hostname == pattern;
        }

        if !pattern.starts_with("*.") || pattern[1..].contains('*') {
            return false;
        }

        if hostname.len() < pattern.len() {
            return false;
        }

        if pattern == "*." {
            return false;
        }

        let suffix = &pattern[1..];
        if !hostname.ends_with(suffix) {
            return false;
        }

        let suffix_start_index_in_hostname = hostname.len() - suffix.len();
        if suffix_start_index_in_hostname > 0 
            && hostname[..suffix_start_index_in_hostname - 1].contains('.') {
            // Note: Kotlin's lastIndexOf('.', index) != -1 is equivalent to contains('.') in the slice
            // but specifically we check if there's a dot before the suffix start.
            // The Kotlin logic: hostname.lastIndexOf('.', suffixStartIndexInHostname - 1) != -1
            // means there is a dot at or before the character preceding the suffix.
            // Since we already know the suffix starts with '.', we are checking for a second dot
            // that would imply the asterisk matched across labels.
            
            // Re-evaluating Kotlin: hostname.lastIndexOf('.', suffixStartIndexInHostname - 1)
            // If hostname is "a.b.example.com." and suffix is ".example.com."
            // suffixStartIndexInHostname is 3 (index of '.'). 
            // lastIndexOf('.', 2) finds the dot at index 1. This returns true -> return false.
            // This correctly prevents *.example.com matching a.b.example.com.
            return false;
        }

        true
    }

    pub fn all_subject_alt_names(&self, certificate: &X509Certificate) -> Vec<String> {
        let alt_ipa_names = self.get_subject_alt_names(certificate, Self::ALT_IPA_NAME);
        let alt_dns_names = self.get_subject_alt_names(certificate, Self::ALT_DNS_NAME);
        let mut all = alt_ipa_names;
        all.extend(alt_dns_names);
        all
    }

    fn get_subject_alt_names(&self, certificate: &X509Certificate, name_type: i32) -> Vec<String> {
        let subject_alt_names = match &certificate.subject_alternative_names {
            Some(names) => names,
            None => return Vec::new(),
        };

        let mut result = Vec::new();
        for subject_alt_name in subject_alt_names {
            if subject_alt_name.len() < 2 {
                continue;
            }
            // The first element is the type (represented as a String in this mock, 
            // but in JVM it's an Integer). We assume the mock stores the type as a stringified int.
            let type_val = subject_alt_name[0].as_ref()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(-1);

            if type_val != name_type {
                continue;
            }

            if let Some(Some(alt_name)) = subject_alt_name.get(1) {
                result.push(alt_name.clone());
            }
        }
        result
    }
}

impl HostnameVerifier for OkHostnameVerifier {
    fn verify(&self, host: &str, session: &SSLSession) -> bool {
        if !host.is_ascii() {
            return false;
        }
        
        // In Rust, we don't have the exact same Exception hierarchy as JVM, 
        // but we preserve the logic of returning false on failure.
        if let Some(cert) = session.peer_certificates.first() {
            self.verify_with_cert(host, cert)
        } else {
            false
        }
    }
}

trait StringExt {
    fn is_ascii(&self) -> bool;
    fn ascii_to_lowercase(&self) -> String;
}

impl StringExt for str {
    fn is_ascii(&self) -> bool {
        self.is_ascii()
    }

    fn ascii_to_lowercase(&self) -> String {
        if self.is_ascii() {
            self.to_lowercase()
        } else {
            self.to_string()
        }
    }
}

impl StringExt for String {
    fn is_ascii(&self) -> bool {
        self.as_str().is_ascii()
    }

    fn ascii_to_lowercase(&self) -> String {
        self.as_str().ascii_to_lowercase()
    }
}