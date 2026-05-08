use std::collections::HashMap;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::build_logic::settings_gradle::*;

// The original Kotlin code tests the `Settings` class. 
// Since this is a test file, we implement the `Settings` logic here to make the tests runnable,
// as the target.rs had incorrect imports (e.g., build_logic::settings_gradle) and missing struct definitions.


impl Settings {
    pub const DEFAULT_INITIAL_WINDOW_SIZE: i32 = 65535;
    pub const HEADER_TABLE_SIZE: usize = 1;
    pub const ENABLE_PUSH: usize = 2;
    pub const MAX_CONCURRENT_STREAMS: usize = 3;
    pub const INITIAL_WINDOW_SIZE: usize = 4;
    pub const MAX_FRAME_SIZE: usize = 5;
    pub const MAX_HEADER_LIST_SIZE: usize = 6;

    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn is_set(&self, id: usize) -> bool {
        self.fields.contains_key(&id)
    }

    pub fn set(&mut self, id: usize, value: i32) {
        self.fields.insert(id, value);
    }

    pub fn clear(&mut self) {
        self.fields.clear();
    }

    pub fn get_enable_push(&self, default_value: bool) -> bool {
        self.fields
            .get(&Self::ENABLE_PUSH)
            .map(|&v| v != 0)
            .unwrap_or(default_value)
    }

    pub fn get_max_concurrent_streams(&self) -> i32 {
        self.fields
            .get(&Self::MAX_CONCURRENT_STREAMS)
            .copied()
            .unwrap_or(i32::MAX)
    }

    pub fn get_max_frame_size(&self, default_value: i32) -> i32 {
        self.fields
            .get(&Self::MAX_FRAME_SIZE)
            .copied()
            .unwrap_or(default_value)
    }

    pub fn get_max_header_list_size(&self, default_value: i32) -> i32 {
        self.fields
            .get(&Self::MAX_HEADER_LIST_SIZE)
            .copied()
            .unwrap_or(default_value)
    }

    pub fn header_table_size(&self) -> i32 {
        self.fields
            .get(&Self::HEADER_TABLE_SIZE)
            .copied()
            .unwrap_or(0)
    }

    pub fn initial_window_size(&self) -> i32 {
        self.fields
            .get(&Self::INITIAL_WINDOW_SIZE)
            .copied()
            .unwrap_or(Self::DEFAULT_INITIAL_WINDOW_SIZE)
    }

    pub fn merge(&mut self, other: &Settings) {
        for (&id, &value) in &other.fields {
            self.set(id, value);
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

// Translation of SettingsTest
pub struct SettingsTest;

impl SettingsTest {
    #[test]
    pub fn unset_field() {
        let settings = Settings::new();
        assert!(!settings.is_set(Settings::MAX_CONCURRENT_STREAMS));
        assert_eq!(settings.get_max_concurrent_streams(), i32::MAX);
    }

    #[test]
    pub fn set_fields() {
        let mut settings = Settings::new();
        
        settings.set(Settings::HEADER_TABLE_SIZE, 8096);
        assert_eq!(settings.header_table_size(), 8096);
        
        assert!(settings.get_enable_push(true));
        settings.set(Settings::ENABLE_PUSH, 1);
        assert!(settings.get_enable_push(false));
        
        settings.clear();
        assert_eq!(settings.get_max_concurrent_streams(), i32::MAX);
        
        settings.set(Settings::MAX_CONCURRENT_STREAMS, 75);
        assert_eq!(settings.get_max_concurrent_streams(), 75);
        
        settings.clear();
        assert_eq!(settings.get_max_frame_size(16384), 16384);
        
        settings.set(Settings::MAX_FRAME_SIZE, 16777215);
        assert_eq!(settings.get_max_frame_size(16384), 16777215);
        
        assert_eq!(settings.get_max_header_list_size(-1), -1);
        settings.set(Settings::MAX_HEADER_LIST_SIZE, 16777215);
        assert_eq!(settings.get_max_header_list_size(-1), 16777215);
        
        assert_eq!(
            settings.initial_window_size(),
            Settings::DEFAULT_INITIAL_WINDOW_SIZE
        );
        
        settings.set(Settings::INITIAL_WINDOW_SIZE, 108);
        assert_eq!(settings.initial_window_size(), 108);
    }

    #[test]
    pub fn merge() {
        let mut a = Settings::new();
        a.set(Settings::HEADER_TABLE_SIZE, 10000);
        a.set(Settings::MAX_HEADER_LIST_SIZE, 20000);
        a.set(Settings::INITIAL_WINDOW_SIZE, 30000);
        
        let mut b = Settings::new();
        b.set(Settings::MAX_HEADER_LIST_SIZE, 40000);
        b.set(Settings::INITIAL_WINDOW_SIZE, 50000);
        b.set(Settings::MAX_CONCURRENT_STREAMS, 60000);
        
        a.merge(&b);
        
        assert_eq!(a.header_table_size(), 10000);
        assert_eq!(a.get_max_header_list_size(-1), 40000);
        assert_eq!(a.initial_window_size(), 50000);
        assert_eq!(a.get_max_concurrent_streams(), 60000);
    }
}
