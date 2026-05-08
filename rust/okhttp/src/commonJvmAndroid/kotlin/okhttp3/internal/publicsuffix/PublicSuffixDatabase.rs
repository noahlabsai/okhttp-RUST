use std::sync::{Mutex, OnceLock};

// Mocking ByteString as it is a dependency from okio.
// In a real production environment, this would be imported from a crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    pub fn of(byte: u8) -> Self {
        ByteString(vec![byte])
    }

    pub fn encode_utf8(s: &str) -> Self {
        ByteString(s.as_bytes().to_vec())
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> u8 {
        self.0[index]
    }

    pub fn substring(&self, start: usize, end: usize) -> String {
        String::from_utf8_lossy(&self.0[start..end]).into_owned()
    }
}

impl std::ops::Index<usize> for ByteString {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

// Mocking PublicSuffixList as it is a dependency.
pub struct PublicSuffixList {
    pub bytes: ByteString,
    pub exception_bytes: ByteString,
}

impl PublicSuffixList {
    pub const Default: PublicSuffixList = PublicSuffixList {
        bytes: ByteString(vec![]),
        exception_bytes: ByteString(vec![]),
    };

    pub fn ensure_loaded(&self) {
        // Implementation provided by PublicSuffixList
    }
}

pub struct PublicSuffixDatabase {
    public_suffix_list: PublicSuffixList,
}

impl PublicSuffixDatabase {
    /// Internal constructor to match Kotlin's internal constructor.
    pub fn new(public_suffix_list: PublicSuffixList) -> Self {
        PublicSuffixDatabase {
            public_suffix_list,
        }
    }

    /// Returns the effective top-level domain plus one (eTLD+1) by referencing the public suffix list.
    /// Returns None if the domain is a public suffix or a private address.
    pub fn get_effective_tld_plus_one(&self, domain: &str) -> Option<String> {
        // In Rust, we use a library like `idna` for IDN.toUnicode. 
        // For this translation, we assume the domain is already handled or use a generated-compatibility.
        let unicode_domain = domain.to_string(); 
        let domain_labels = self.split_domain(&unicode_domain);

        let rule = self.find_matching_rule(&domain_labels);
        if domain_labels.len() == rule.len() && rule.get(0).and_then(|s| s.chars().next()) != Some('!') {
            return None; // The domain is a public suffix.
        }

        let first_label_offset = if rule.get(0).and_then(|s| s.chars().next()) == Some('!') {
            // Exception rules hold the effective TLD plus one.
            domain_labels.len() - rule.len()
        } else {
            // Otherwise the rule is for a public suffix, so we must take one more label.
            domain_labels.len() - (rule.len() + 1)
        };

        let labels = self.split_domain(domain);
        let result = labels.into_iter().skip(first_label_offset).collect::<Vec<_>>().join(".");
        Some(result)
    }

    fn split_domain(&self, domain: &str) -> Vec<String> {
        let mut domain_labels: Vec<String> = domain.split('.').map(|s| s.to_string()).collect();

        if domain_labels.last() == Some(&"".to_string()) {
            // allow for domain name trailing dot
            domain_labels.pop();
        }

        domain_labels
    }

    fn find_matching_rule(&self, domain_labels: &[String]) -> Vec<String> {
        self.public_suffix_list.ensure_loaded();

        // Break apart the domain into UTF-8 labels.
        let domain_labels_utf8_bytes: Vec<ByteString> = domain_labels
            .iter()
            .map(|l| ByteString::encode_utf8(l))
            .collect();

        // Exact matches
        let mut exact_match: Option<String> = None;
        for i in 0..domain_labels_utf8_bytes.len() {
            if let Some(rule) = binary_search(&self.public_suffix_list.bytes, &domain_labels_utf8_bytes, i) {
                exact_match = Some(rule);
                break;
            }
        }

        // Wildcard matches
        let mut wildcard_match: Option<String> = None;
        if domain_labels_utf8_bytes.len() > 1 {
            let mut labels_with_wildcard = domain_labels_utf8_bytes.clone();
            for label_index in 0..labels_with_wildcard.len() - 1 {
                labels_with_wildcard[label_index] = ByteString::of(b'*');
                if let Some(rule) = binary_search(&self.public_suffix_list.bytes, &labels_with_wildcard, label_index) {
                    wildcard_match = Some(rule);
                    break;
                }
            }
        }

        // Exception rules
        let mut exception: Option<String> = None;
        if wildcard_match.is_some() {
            for label_index in 0..domain_labels_utf8_bytes.len() - 1 {
                if let Some(rule) = binary_search(&self.public_suffix_list.exception_bytes, &domain_labels_utf8_bytes, label_index) {
                    exception = Some(rule);
                    break;
                }
            }
        }

        if let Some(exc) = exception {
            let formatted = format!("!{}", exc);
            return formatted.split('.').map(|s| s.to_string()).collect();
        } else if exact_match.is_none() && wildcard_match.is_none() {
            return vec!["*".to_string()];
        }

        let exact_rule_labels: Vec<String> = exact_match.map(|s| s.split('.').map(|x| x.to_string()).collect()).unwrap_or_default();
        let wildcard_rule_labels: Vec<String> = wildcard_match.map(|s| s.split('.').map(|x| x.to_string()).collect()).unwrap_or_default();

        if exact_rule_labels.len() > wildcard_rule_labels.len() {
            exact_rule_labels
        } else {
            wildcard_rule_labels
        }
    }

    pub fn get() -> &'static Mutex<PublicSuffixDatabase> {
        static INSTANCE: OnceLock<Mutex<PublicSuffixDatabase>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Mutex::new(PublicSuffixDatabase::new(PublicSuffixList::Default))
        })
    }

    pub fn reset_for_tests() {
        if let Ok(mut instance) = Self::get().lock() {
            *instance = PublicSuffixDatabase::new(PublicSuffixList::Default);
        }
    }
}

fn binary_search(data: &ByteString, labels: &[ByteString], label_index: usize) -> Option<String> {
    let mut low = 0;
    let mut high = data.size();
    let mut match_found: Option<String> = None;

    while low < high {
        let mut mid = (low + high) / 2;
        
        // Search for a '\n' that marks the start of a value.
        while mid > 0 && data[mid] != b'\n' {
            mid -= 1;
        }
        mid += 1;

        // Now look for the ending '\n'.
        let mut end = 1;
        while mid + end < data.size() && data[mid + end] != b'\n' {
            end += 1;
        }
        let public_suffix_length = end;

        let mut compare_result: i32 = 0;
        let mut current_label_index = label_index;
        let mut current_label_byte_index = 0;
        let mut public_suffix_byte_index = 0;
        let mut expect_dot = false;

        loop {
            let byte0: u8;
            if expect_dot {
                byte0 = b'.';
                expect_dot = false;
            } else {
                if current_label_index >= labels.len() {
                    break;
                }
                byte0 = labels[current_label_index][current_label_byte_index];
            }

            if mid + public_suffix_byte_index >= data.size() {
                compare_result = 1;
                break;
            }
            let byte1 = data[mid + public_suffix_byte_index];

            compare_result = (byte0 as i32) - (byte1 as i32);
            if compare_result != 0 {
                break;
            }

            public_suffix_byte_index += 1;
            current_label_byte_index += 1;
            if public_suffix_byte_index == public_suffix_length {
                break;
            }

            if current_label_index < labels.len() && labels[current_label_index].size() == current_label_byte_index {
                if current_label_index == labels.len() - 1 {
                    break;
                } else {
                    current_label_index += 1;
                    current_label_byte_index = 0;
                    expect_dot = true;
                }
            }
        }

        if compare_result < 0 {
            if mid == 0 { break; }
            high = mid - 1;
        } else if compare_result > 0 {
            low = mid + end + 1;
        } else {
            let public_suffix_bytes_left = public_suffix_length - public_suffix_byte_index;
            let mut label_bytes_left = if current_label_index < labels.len() {
                labels[current_label_index].size() - current_label_byte_index
            } else {
                0
            };
            for i in (current_label_index + 1)..labels.len() {
                label_bytes_left += labels[i].size();
            }

            if label_bytes_left < public_suffix_bytes_left {
                if mid == 0 { break; }
                high = mid - 1;
            } else if label_bytes_left > public_suffix_bytes_left {
                low = mid + end + 1;
            } else {
                match_found = Some(data.substring(mid, mid + public_suffix_length));
                break;
            }
        }
    }
    match_found
}
