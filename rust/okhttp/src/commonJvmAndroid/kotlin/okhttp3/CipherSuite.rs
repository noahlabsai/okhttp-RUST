use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

// [TLS cipher suites][iana_tls_parameters].
//
// **Not all cipher suites are supported on all platforms.** As newer cipher suites are created (for
// stronger privacy, better performance, etc.) they will be adopted by the platform and then exposed
// here. Cipher suites that are not available on either Android (through API level 24) or Java
// (through JDK 9) are omitted for brevity.
//
// See [Android SSLEngine][sslengine] which lists the cipher suites supported by Android.
//
// See [JDK Providers][oracle_providers] which lists the cipher suites supported by Oracle.
//
// See [NativeCrypto.java][conscrypt_providers] which lists the cipher suites supported by
// Conscrypt.
//
// [iana_tls_parameters]: https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml
// [sslengine]: https://developer.android.com/reference/javax/net/ssl/SSLEngine.html
// [oracle_providers]: https://docs.oracle.com/javase/10/security/oracle-providers.htm
// [conscrypt_providers]: https://github.com/google/conscrypt/blob/master/common/src/main/java/org/conscrypt/NativeCrypto.java
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CipherSuite {
    // Returns the Java name of this cipher suite. For some older cipher suites the Java name has the
    // prefix `SSL_`, causing the Java name to be different from the instance name which is always
    // prefixed `TLS_`. For example, `TLS_RSA_EXPORT_WITH_RC4_40_MD5.java_name()` is
    // `"SSL_RSA_EXPORT_WITH_RC4_40_MD5"`.
    pub java_name: String,
}

impl CipherSuite {
    // Deprecated: moved to field java_name
    pub fn java_name_method(&self) -> String {
        self.java_name.clone()
    }
}

impl std::fmt::Display for CipherSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.java_name)
    }
}

// Compares cipher suites names like "TLS_RSA_WITH_NULL_MD5" and "SSL_RSA_WITH_NULL_MD5",
// ignoring the "TLS_" or "SSL_" prefix which is not consistent across platforms.
pub struct OrderByName;

impl OrderByName {
    pub fn compare(a: &str, b: &str) -> i32 {
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        let mut i = 4;
        let limit = a_bytes.len().min(b_bytes.len());
        while i < limit {
            let char_a = a_bytes[i];
            let char_b = b_bytes[i];
            if char_a != char_b {
                return if char_a < char_b { -1 } else { 1 };
            }
            i += 1;
        }
        let length_a = a_bytes.len();
        let length_b = b_bytes.len();
        if length_a != length_b {
            return if length_a < length_b { -1 } else { 1 };
        }
        0
    }
}

static INSTANCES: OnceLock<Mutex<HashMap<String, Arc<CipherSuite>>>> = OnceLock::new();

fn get_instances() -> &'static Mutex<HashMap<String, Arc<CipherSuite>>> {
    INSTANCES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn init(java_name: &str, _value: i32) -> Arc<CipherSuite> {
    let suite = Arc::new(CipherSuite {
        java_name: java_name.to_string(),
    });
    let mut instances = get_instances().lock().unwrap();
    instances.insert(java_name.to_string(), Arc::clone(&suite));
    suite
}

fn secondary_name(java_name: &str) -> String {
    if java_name.starts_with("TLS_") {
        format!("SSL_{}", &java_name[4..])
    } else if java_name.starts_with("SSL_") {
        format!("TLS_{}", &java_name[4..])
    } else {
        java_name.to_string()
    }
}

impl CipherSuite {
    pub fn for_java_name(java_name: String) -> Arc<CipherSuite> {
        let mut instances = get_instances().lock().unwrap();
        if let Some(result) = instances.get(&java_name) {
            return Arc::clone(result);
        }

        let secondary = secondary_name(&java_name);
        let result = if let Some(res) = instances.get(&secondary) {
            Arc::clone(res)
        } else {
            Arc::new(CipherSuite {
                java_name: java_name.clone(),
            })
        };

        instances.insert(java_name, Arc::clone(&result));
        result
    }
}

// Static instances
pub static TLS_RSA_WITH_NULL_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_NULL_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_EXPORT_WITH_RC4_40_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_RC4_128_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_EXPORT_WITH_DES40_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_DES_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_EXPORT_WITH_DES40_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_DES_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_EXPORT_WITH_DES40_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_DES_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_EXPORT_WITH_RC4_40_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_RC4_128_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_EXPORT_WITH_DES40_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_DES_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_WITH_DES_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_WITH_DES_CBC_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_WITH_3DES_EDE_CBC_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_WITH_RC4_128_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_EXPORT_WITH_DES_CBC_40_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_EXPORT_WITH_RC4_40_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_EXPORT_WITH_DES_CBC_40_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_KRB5_EXPORT_WITH_RC4_40_MD5: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_NULL_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_AES_256_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_CAMELLIA_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_CAMELLIA_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_CAMELLIA_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_AES_256_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_AES_256_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_AES_256_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_CAMELLIA_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_CAMELLIA_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_CAMELLIA_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_PSK_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_PSK_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_PSK_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_PSK_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_SEED_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_RSA_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_DSS_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DH_anon_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_EMPTY_RENEGOTIATION_INFO_SCSV: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_FALLBACK_SCSV: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_NULL_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_NULL_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_NULL_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_NULL_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_anon_WITH_NULL_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_anon_WITH_RC4_128_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_anon_WITH_3DES_EDE_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_anon_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_anon_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_AES_256_CBC_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_AES_128_CBC_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_AES_256_CBC_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_ECDSA_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDH_RSA_WITH_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_PSK_WITH_AES_128_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_PSK_WITH_AES_256_CBC_SHA: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_DHE_RSA_WITH_CHACHA20_POLY1305_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_ECDHE_PSK_WITH_CHACHA20_POLY1305_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_AES_128_GCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_AES_256_GCM_SHA384: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_CHACHA20_POLY1305_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_AES_128_CCM_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();
pub static TLS_AES_128_CCM_8_SHA256: OnceLock<Arc<CipherSuite>> = OnceLock::new();

// Helper to initialize the static constants. In a real Rust app, these would be 
// initialized via a lazy_static or OnceLock during first access.
pub fn initialize_cipher_suites() {
    let _ = TLS_RSA_WITH_NULL_MD5.set(init("SSL_RSA_WITH_NULL_MD5", 0x0001));
    let _ = TLS_RSA_WITH_NULL_SHA.set(init("SSL_RSA_WITH_NULL_SHA", 0x0002));
    let _ = TLS_RSA_EXPORT_WITH_RC4_40_MD5.set(init("SSL_RSA_EXPORT_WITH_RC4_40_MD5", 0x0003));
    let _ = TLS_RSA_WITH_RC4_128_MD5.set(init("SSL_RSA_WITH_RC4_128_MD5", 0x0004));
    let _ = TLS_RSA_WITH_RC4_128_SHA.set(init("SSL_RSA_WITH_RC4_128_SHA", 0x0005));
    let _ = TLS_RSA_EXPORT_WITH_DES40_CBC_SHA.set(init("SSL_RSA_EXPORT_WITH_DES40_CBC_SHA", 0x0008));
    let _ = TLS_RSA_WITH_DES_CBC_SHA.set(init("SSL_RSA_WITH_DES_CBC_SHA", 0x0009));
    let _ = TLS_RSA_WITH_3DES_EDE_CBC_SHA.set(init("SSL_RSA_WITH_3DES_EDE_CBC_SHA", 0x000a));
    let _ = TLS_DHE_DSS_EXPORT_WITH_DES40_CBC_SHA.set(init("SSL_DHE_DSS_EXPORT_WITH_DES40_CBC_SHA", 0x0011));
    let _ = TLS_DHE_DSS_WITH_DES_CBC_SHA.set(init("SSL_DHE_DSS_WITH_DES_CBC_SHA", 0x0012));
    let _ = TLS_DHE_DSS_WITH_3DES_EDE_CBC_SHA.set(init("SSL_DHE_DSS_WITH_3DES_EDE_CBC_SHA", 0x0013));
    let _ = TLS_DHE_RSA_EXPORT_WITH_DES40_CBC_SHA.set(init("SSL_DHE_RSA_EXPORT_WITH_DES40_CBC_SHA", 0x0014));
    let _ = TLS_DHE_RSA_WITH_DES_CBC_SHA.set(init("SSL_DHE_RSA_WITH_DES_CBC_SHA", 0x0015));
    let _ = TLS_DHE_RSA_WITH_3DES_EDE_CBC_SHA.set(init("SSL_DHE_RSA_WITH_3DES_EDE_CBC_SHA", 0x0016));
    let _ = TLS_DH_anon_EXPORT_WITH_RC4_40_MD5.set(init("SSL_DH_anon_EXPORT_WITH_RC4_40_MD5", 0x0017));
    let _ = TLS_DH_anon_WITH_RC4_128_MD5.set(init("SSL_DH_anon_WITH_RC4_128_MD5", 0x0018));
    let _ = TLS_DH_anon_EXPORT_WITH_DES40_CBC_SHA.set(init("SSL_DH_anon_EXPORT_WITH_DES40_CBC_SHA", 0x0019));
    let _ = TLS_DH_anon_WITH_DES_CBC_SHA.set(init("SSL_DH_anon_WITH_DES_CBC_SHA", 0x001a));
    let _ = TLS_DH_anon_WITH_3DES_EDE_CBC_SHA.set(init("SSL_DH_anon_WITH_3DES_EDE_CBC_SHA", 0x001b));
    let _ = TLS_KRB5_WITH_DES_CBC_SHA.set(init("TLS_KRB5_WITH_DES_CBC_SHA", 0x001e));
    let _ = TLS_KRB5_WITH_3DES_EDE_CBC_SHA.set(init("TLS_KRB5_WITH_3DES_EDE_CBC_SHA", 0x001f));
    let _ = TLS_KRB5_WITH_RC4_128_SHA.set(init("TLS_KRB5_WITH_RC4_128_SHA", 0x0020));
    let _ = TLS_KRB5_WITH_DES_CBC_MD5.set(init("TLS_KRB5_WITH_DES_CBC_MD5", 0x0022));
    let _ = TLS_KRB5_WITH_3DES_EDE_CBC_MD5.set(init("TLS_KRB5_WITH_3DES_EDE_CBC_MD5", 0x0023));
    let _ = TLS_KRB5_WITH_RC4_128_MD5.set(init("TLS_KRB5_WITH_RC4_128_MD5", 0x0024));
    let _ = TLS_KRB5_EXPORT_WITH_DES_CBC_40_SHA.set(init("TLS_KRB5_EXPORT_WITH_DES_CBC_40_SHA", 0x0026));
    let _ = TLS_KRB5_EXPORT_WITH_RC4_40_SHA.set(init("TLS_KRB5_EXPORT_WITH_RC4_40_SHA", 0x0028));
    let _ = TLS_KRB5_EXPORT_WITH_DES_CBC_40_MD5.set(init("TLS_KRB5_EXPORT_WITH_DES_CBC_40_MD5", 0x0029));
    let _ = TLS_KRB5_EXPORT_WITH_RC4_40_MD5.set(init("TLS_KRB5_EXPORT_WITH_RC4_40_MD5", 0x002b));
    let _ = TLS_RSA_WITH_AES_128_CBC_SHA.set(init("TLS_RSA_WITH_AES_128_CBC_SHA", 0x002f));
    let _ = TLS_DHE_DSS_WITH_AES_128_CBC_SHA.set(init("TLS_DHE_DSS_WITH_AES_128_CBC_SHA", 0x0032));
    let _ = TLS_DHE_RSA_WITH_AES_128_CBC_SHA.set(init("TLS_DHE_RSA_WITH_AES_128_CBC_SHA", 0x0033));
    let _ = TLS_DH_anon_WITH_AES_128_CBC_SHA.set(init("TLS_DH_anon_WITH_AES_128_CBC_SHA", 0x0034));
    let _ = TLS_RSA_WITH_AES_256_CBC_SHA.set(init("TLS_RSA_WITH_AES_256_CBC_SHA", 0x0035));
    let _ = TLS_DHE_DSS_WITH_AES_256_CBC_SHA.set(init("TLS_DHE_DSS_WITH_AES_256_CBC_SHA", 0x0038));
    let _ = TLS_DHE_RSA_WITH_AES_256_CBC_SHA.set(init("TLS_DHE_RSA_WITH_AES_256_CBC_SHA", 0x0039));
    let _ = TLS_DH_anon_WITH_AES_256_CBC_SHA.set(init("TLS_DH_anon_WITH_AES_256_CBC_SHA", 0x003a));
    let _ = TLS_RSA_WITH_NULL_SHA256.set(init("TLS_RSA_WITH_NULL_SHA256", 0x003b));
    let _ = TLS_RSA_WITH_AES_128_CBC_SHA256.set(init("TLS_RSA_WITH_AES_128_CBC_SHA256", 0x003c));
    let _ = TLS_RSA_WITH_AES_256_CBC_SHA256.set(init("TLS_RSA_WITH_AES_256_CBC_SHA256", 0x003d));
    let _ = TLS_DHE_DSS_WITH_AES_128_CBC_SHA256.set(init("TLS_DHE_DSS_WITH_AES_128_CBC_SHA256", 0x0040));
    let _ = TLS_RSA_WITH_CAMELLIA_128_CBC_SHA.set(init("TLS_RSA_WITH_CAMELLIA_128_CBC_SHA", 0x0041));
    let _ = TLS_DHE_DSS_WITH_CAMELLIA_128_CBC_SHA.set(init("TLS_DHE_DSS_WITH_CAMELLIA_128_CBC_SHA", 0x0044));
    let _ = TLS_DHE_RSA_WITH_CAMELLIA_128_CBC_SHA.set(init("TLS_DHE_RSA_WITH_CAMELLIA_128_CBC_SHA", 0x0045));
    let _ = TLS_DHE_RSA_WITH_AES_128_CBC_SHA256.set(init("TLS_DHE_RSA_WITH_AES_128_CBC_SHA256", 0x0067));
    let _ = TLS_DHE_DSS_WITH_AES_256_CBC_SHA256.set(init("TLS_DHE_DSS_WITH_AES_256_CBC_SHA256", 0x006a));
    let _ = TLS_DHE_RSA_WITH_AES_256_CBC_SHA256.set(init("TLS_DHE_RSA_WITH_AES_256_CBC_SHA256", 0x006b));
    let _ = TLS_DH_anon_WITH_AES_128_CBC_SHA256.set(init("TLS_DH_anon_WITH_AES_128_CBC_SHA256", 0x006c));
    let _ = TLS_DH_anon_WITH_AES_256_CBC_SHA256.set(init("TLS_DH_anon_WITH_AES_256_CBC_SHA256", 0x006d));
    let _ = TLS_RSA_WITH_CAMELLIA_256_CBC_SHA.set(init("TLS_RSA_WITH_CAMELLIA_256_CBC_SHA", 0x0084));
})}
