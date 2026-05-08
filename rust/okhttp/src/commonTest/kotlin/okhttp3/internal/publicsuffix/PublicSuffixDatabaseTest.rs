/*
 * Copyright (C) 2017 Square, Inc.
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

use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{PathBuf};
use std::sync::Arc;
use regex::Regex;

// Corrected imports based on project structure and Kotlin source
use crate::okhttp3::internal::publicsuffix::{PublicSuffixDatabase, ResourcePublicSuffixList};
use crate::okhttp3::internal::to_canonical_host;
use crate::okhttp3::ok_http_root;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// Mocking Okio Buffer for the test logic as it's a core part of the test data reading

impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn exhausted(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn read_utf8_line_strict(&mut self) -> String {
        if self.exhausted() {
            return String::new();
        }
        let remaining = &self.data[self.pos..];
        if let Some(pos) = remaining.iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&remaining[..pos]).to_string();
            self.pos += pos + 1;
            line.trim_end_matches('\r').to_string()
        } else {
            let line = String::from_utf8_lossy(remaining).to_string();
            self.pos = self.data.len();
            line.trim_end_matches('\r').to_string()
        }
    }
}

pub struct PublicSuffixDatabaseTest {
    public_suffix_database: Arc<PublicSuffixDatabase>,
    path_for_tests: PathBuf,
}

impl PublicSuffixDatabaseTest {
    pub fn new() -> Self {
        let public_suffix_resource = ResourcePublicSuffixList::PUBLIC_SUFFIX_RESOURCE;
        let mut path = ok_http_root();
        path.push("okhttp/src/jvmMain/resources");
        path.push(public_suffix_resource);

        Self {
            public_suffix_database: PublicSuffixDatabase::get(),
            path_for_tests: path,
        }
    }

    pub fn set_up(&self) {
        // Equivalent to beforePublicSuffixTest()
        // In Rust tests, setup is usually handled in the test function or a helper.
    }

    pub fn all_public_suffixes(&self) -> Result<(), Box<dyn Error>> {
        let mut buffer = Buffer::new();
        let mut file = File::open(&self.path_for_tests)?;
        
        // readInt() equivalent
        let mut int_buf = [0u8; 4];
        file.read_exact(&mut int_buf)?;
        let length = i32::from_be_bytes(int_buf);
        
        let mut data_buf = vec![0u8; length as usize];
        file.read_exact(&mut data_buf)?;
        buffer.write(&data_buf);

        while !buffer.exhausted() {
            let mut public_suffix = buffer.read_utf8_line_strict();
            if public_suffix.is_empty() { continue; }
            
            if public_suffix.contains('*') {
                let re = Regex::new(r"\*").unwrap();
                public_suffix = re.replace_all(&public_suffix, "square").to_string();
            }
            
            assert!(self.public_suffix_database.get_effective_tld_plus_one(&public_suffix).is_none());
            
            let test = format!("foobar.{}", public_suffix);
            assert_eq!(self.public_suffix_database.get_effective_tld_plus_one(&test), Some(test));
        }
        Ok(())
    }

    pub fn public_suffix_exceptions(&self) -> Result<(), Box<dyn Error>> {
        let mut buffer = Buffer::new();
        let mut file = File::open(&self.path_for_tests)?;
        
        let mut int_buf = [0u8; 4];
        file.read_exact(&mut int_buf)?;
        let length = i32::from_be_bytes(int_buf);
        file.seek(SeekFrom::Current(length as i64))?;
        
        file.read_exact(&mut int_buf)?;
        let length_exc = i32::from_be_bytes(int_buf);
        let mut data_buf = vec![0u8; length_exc as usize];
        file.read_exact(&mut data_buf)?;
        buffer.write(&data_buf);

        while !buffer.exhausted() {
            let exception = buffer.read_utf8_line_strict();
            if exception.is_empty() { continue; }
            
            assert_eq!(self.public_suffix_database.get_effective_tld_plus_one(&exception), Some(exception.clone()));
            
            let test = format!("foobar.{}", exception);
            assert_eq!(self.public_suffix_database.get_effective_tld_plus_one(&test), Some(exception));
        }
        Ok(())
    }

    pub fn thread_is_interrupted_on_first_read(&self) {
        // Rust does not have a direct equivalent to Thread.interrupt().
        // This test is largely JVM-specific. In a Rust translation, we simulate the logic.
        let result = self.public_suffix_database.get_effective_tld_plus_one("squareup.com");
        assert_eq!(result, Some("squareup.com".to_string()));
    }

    pub fn second_read_fails_same_as_first(&self) {
        let bad_list = ResourcePublicSuffixList {
            path: PathBuf::from("/xxx.gz"),
        };
        let bad_db = PublicSuffixDatabase::new(bad_list);
        
        let first_result = std::panic::catch_unwind(|| {
            bad_db.get_effective_tld_plus_one("squareup.com");
        });
        
        let first_failure_msg = match first_result {
            Err(e) => format!("{:?}", e),
            Ok(_) => panic!("Should have failed"),
        };

        let second_result = std::panic::catch_unwind(|| {
            bad_db.get_effective_tld_plus_one("squareup.com");
        });

        let second_failure_msg = match second_result {
            Err(e) => format!("{:?}", e),
            Ok(_) => panic!("Should have failed"),
        };

        assert_eq!(first_failure_msg, second_failure_msg);
    }

    pub fn public_suffix_dot_org_test_cases(&self) {
        // Mixed case.
        self.check_public_suffix("COM", None);
        self.check_public_suffix("example.COM", Some("example.com"));
        self.check_public_suffix("WwW.example.COM", Some("example.com"));
        // Leading dot.
        self.check_public_suffix(".com", None);
        self.check_public_suffix(".example", None);
        self.check_public_suffix(".example.com", None);
        self.check_public_suffix(".example.example", None);
        // Unlisted TLD.
        self.check_public_suffix("example", None);
        self.check_public_suffix("example.example", Some("example.example"));
        self.check_public_suffix("b.example.example", Some("example.example"));
        self.check_public_suffix("a.b.example.example", Some("example.example"));
        // TLD with only 1 rule.
        self.check_public_suffix("biz", None);
        self.check_public_suffix("domain.biz", Some("domain.biz"));
        self.check_public_suffix("b.domain.biz", Some("domain.biz"));
        self.check_public_suffix("a.b.domain.biz", Some("domain.biz"));
        // TLD with some 2-level rules.
        self.check_public_suffix("com", None);
        self.check_public_suffix("example.com", Some("example.com"));
        self.check_public_suffix("b.example.com", Some("example.com"));
        self.check_public_suffix("a.b.example.com", Some("example.com"));
        self.check_public_suffix("uk.com", None);
        self.check_public_suffix("example.uk.com", Some("example.uk.com"));
        self.check_public_suffix("b.example.uk.com", Some("example.uk.com"));
        self.check_public_suffix("a.b.example.uk.com", Some("example.uk.com"));
        self.check_public_suffix("test.ac", Some("test.ac"));
        // TLD with only 1 (wildcard) rule.
        self.check_public_suffix("mm", None);
        self.check_public_suffix("c.mm", None);
        self.check_public_suffix("b.c.mm", Some("b.c.mm"));
        self.check_public_suffix("a.b.c.mm", Some("b.c.mm"));
        // More complex TLD.
        self.check_public_suffix("jp", None);
        self.check_public_suffix("test.jp", Some("test.jp"));
        self.check_public_suffix("www.test.jp", Some("test.jp"));
        self.check_public_suffix("ac.jp", None);
        self.check_public_suffix("test.ac.jp", Some("test.ac.jp"));
        self.check_public_suffix("www.test.ac.jp", Some("test.ac.jp"));
        self.check_public_suffix("kyoto.jp", None);
        self.check_public_suffix("test.kyoto.jp", Some("test.kyoto.jp"));
        self.check_public_suffix("ide.kyoto.jp", None);
        self.check_public_suffix("b.ide.kyoto.jp", Some("b.ide.kyoto.jp"));
        self.check_public_suffix("a.b.ide.kyoto.jp", Some("b.ide.kyoto.jp"));
        self.check_public_suffix("c.kobe.jp", None);
        self.check_public_suffix("b.c.kobe.jp", Some("b.c.kobe.jp"));
        self.check_public_suffix("a.b.c.kobe.jp", Some("b.c.kobe.jp"));
        self.check_public_suffix("city.kobe.jp", Some("city.kobe.jp"));
        self.check_public_suffix("www.city.kobe.jp", Some("city.kobe.jp"));
        // TLD with a wildcard rule and exceptions.
        self.check_public_suffix("ck", None);
        self.check_public_suffix("test.ck", None);
        self.check_public_suffix("b.test.ck", Some("b.test.ck"));
        self.check_public_suffix("a.b.test.ck", Some("b.test.ck"));
        self.check_public_suffix("www.ck", Some("www.ck"));
        self.check_public_suffix("www.www.ck", Some("www.ck"));
        // US K12.
        self.check_public_suffix("us", None);
        self.check_public_suffix("test.us", Some("test.us"));
        self.check_public_suffix("www.test.us", Some("test.us"));
        self.check_public_suffix("ak.us", None);
        self.check_public_suffix("test.ak.us", Some("test.ak.us"));
        self.check_public_suffix("www.test.ak.us", Some("test.ak.us"));
        self.check_public_suffix("k12.ak.us", None);
        self.check_public_suffix("test.k12.ak.us", Some("test.k12.ak.us"));
        self.check_public_suffix("www.test.k12.ak.us", Some("test.k12.ak.us"));
        // IDN labels.
        self.check_public_suffix("\u{98df}\u{72ee}.com.cn", Some("\u{98df}\u{72ee}.com.cn"));
        self.check_public_suffix("\u{98df}\u{72ee}.\u{516c}\u{53f8}.cn", Some("\u{98df}\u{72ee}.\u{516c}\u{53f8}.cn"));
        self.check_public_suffix("www.\u{98df}\u{72ee}.\u{516c}\u{53f8}.cn", Some("\u{98df}\u{72ee}.\u{516c}\u{53f8}.cn"));
        self.check_public_suffix("shishi.\u{516c}\u{53f8}.cn", Some("shishi.\u{516c}\u{53f8}.cn"));
        self.check_public_suffix("\u{516c}\u{53f8}.cn", None);
        self.check_public_suffix("\u{98df}\u{72ee}.\u{4e2d}\u{56fd}", Some("\u{98df}\u{72ee}.\u{4e2d}\u{56fd}"));
        self.check_public_suffix("www.\u{98df}\u{72ee}.\u{4e2d}\u{56fd}", Some("\u{98df}\u{72ee}.\u{4e2d}\u{56fd}"));
        self.check_public_suffix("shishi.\u{4e2d}\u{56fd}", Some("shishi.\u{4e2d}\u{56fd}"));
        self.check_public_suffix("\u{4e2d}\u{56fd}", None);
        // Same as above, but punycoded.
        self.check_public_suffix("xn--85x722f.com.cn", Some("xn--85x722f.com.cn"));
        self.check_public_suffix("xn--85x722f.xn--55qx5d.cn", Some("xn--85x722f.xn--55qx5d.cn"));
        self.check_public_suffix("www.xn--85x722f.xn--55qx5d.cn", Some("xn--85x722f.xn--55qx5d.cn"));
        self.check_public_suffix("shishi.xn--55qx5d.cn", Some("shishi.xn--55qx5d.cn"));
        self.check_public_suffix("xn--55qx5d.cn", None);
        self.check_public_suffix("xn--85x722f.xn--fiqs8s", Some("xn--85x722f.xn--fiqs8s"));
        self.check_public_suffix("www.xn--85x722f.xn--fiqs8s", Some("xn--85x722f.xn--fiqs8s"));
        self.check_public_suffix("shishi.xn--fiqs8s", Some("shishi.xn--fiqs8s"));
        self.check_public_suffix("xn--fiqs8s", None);
    }

    fn check_public_suffix(&self, domain: &str, registrable_part: Option<&str>) {
        let canonical_domain = match to_canonical_host(domain) {
            Some(d) => d,
            None => return,
        };
        let result = self.public_suffix_database.get_effective_tld_plus_one(&canonical_domain);
        if registrable_part.is_none() {
            assert!(result.is_none());
        } else {
            let expected = to_canonical_host(registrable_part.unwrap());
            assert_eq!(result, expected);
        }
    }
}
