/*
 * Copyright (C) 2014 Square, Inc.
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

use okio::{BufferedSource, ByteString};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;

// Data class representing a Story in HPACK tests.
// Since the original Kotlin code uses Moshi for JSON, we use serde in Rust.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Story {
    pub file_name: Option<String>,
    // Other fields would be defined here based on the Story class definition
    // which was not provided in the snippet but is implied by the usage.
    #[serde(flatten)]
    pub other_fields: serde_json::Value,
}

impl Story {
    pub const MISSING: Story = Story {
        file_name: None,
        other_fields: serde_json::Value::Null,
    };
}

// Utilities for reading HPACK tests.
pub struct HpackJsonUtil;

impl HpackJsonUtil {
    // Helper to read a Story from a BufferedSource.
    // In Rust, we typically read the source into a string or bytes and then deserialize.
    fn read_story<R: Read>(mut source: R) -> Result<Story, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        source.read_to_end(&mut buffer)?;
        let story: Story = serde_json::from_slice(&buffer)?;
        Ok(story)
    }

    // Helper to read a Story from a Path.
    fn read_story_from_path(path: &Path) -> Result<Story, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        Self::read_story(file)
    }

    /* Iterate through the hpack-test-case resources, only picking stories for the current draft. */
    pub fn stories_for_current_draft() -> Vec<String> {
        // In Rust, resources are typically handled via include_dir or similar crates,
        // or by looking at the relative path from the executable.
        // Here we simulate the logic of looking into a directory.
        let resource_path = Path::new("src/main/resources/hpack-test-case");
        
        if !resource_path.exists() {
            return Vec::new();
        }

        let mut result = Vec::new();
        if let Ok(entries) = fs::read_dir(resource_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let story00 = path.join("story_00.json");
                    if !story00.exists() {
                        continue;
                    }
                    
                    // Try to read the story; if it fails (IOException), skip this path.
                    if Self::read_story_from_path(&story00).is_ok() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            result.push(name.to_string());
                        }
                    }
                }
            }
        }
        result
    }

    /*
     * Reads stories named "story_xx.json" from the folder provided.
     */
    pub fn read_stories(test_folder_name: &str) -> Vec<Story> {
        let mut result = Vec::new();
        let mut i = 0;

        loop {
            // Format: /hpack-test-case/%s/story_%02d.json
            let story_resource_name = format!(
                "src/main/resources/hpack-test-case/{}/story_{:02}.json",
                test_folder_name, i
            );
            
            let path = Path::new(&story_resource_name);
            if !path.exists() {
                break;
            }

            match fs::File::open(path) {
                Ok(file) => {
                    match Self::read_story(file) {
                        Ok(mut story) => {
                            story.file_name = Some(story_resource_name);
                            result.push(story);
                            i += 1;
                        }
                        Err(_) => break, // Stop if we can't parse the story
                    }
                }
                Err(_) => break, // Stop if file not found or inaccessible
            }
        }

        if result.is_empty() {
            // missing files
            result.push(Story::MISSING);
        }

        result
    }
}

// Implementation of ByteString hex conversion to match the Moshi adapter logic.
trait ByteStringExt {
    fn to_hex(&self) -> String;
    fn from_hex(hex: &str) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
}

impl ByteStringExt for ByteString {
    fn to_hex(&self) -> String {
        self.hex()
    }
    fn from_hex(hex: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(ByteString::decode_hex(hex)?)
    }
}