use std::any::Any;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::SimpleProvider::*;
use crate::okhttp_hpacktests::src::test::java::okhttp3::internal::http2::hpackjson::HpackJsonUtil::*;
use crate::okhttp_hpacktests::src::test::java::okhttp3::internal::http2::hpackjson::Story::*;
use crate::okhttp_hpacktests::src::test::java::okhttp3::internal::http2::HpackDecodeTestBase::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;

// HpackDecodeInteropTest translates the Kotlin HpackDecodeInteropTest class.
// It inherits from HpackDecodeTestBase.
pub struct HpackDecodeInteropTest {
    base: HpackDecodeTestBase,
}

impl HpackDecodeInteropTest {
    pub fn new() -> Self {
        Self {
            base: HpackDecodeTestBase::new(),
        }
    }

    // Translates `testGoodDecoderInterop(story: Story)`
    // In Rust, the @ParameterizedTest logic is typically handled by the test runner
    // calling this method for each story provided by the StoriesTestProvider.
    pub fn test_good_decoder_interop(&mut self, story: Story) {
        // assumeFalse(story === Story.MISSING, "...")
        // In Rust, we compare the story with the MISSING constant.
        // Since Story is a data class (struct), we check for equality.
        if story == Story::missing() {
            // In a real JUnit environment, assumeFalse would skip the test.
            // Here we panic or return to simulate the assumption failure.
            panic!("Test stories missing, checkout git submodule");
        }

        // Call the inherited method from HpackDecodeTestBase
        self.base.test_decoder(story);
    }
}

// Internal class StoriesTestProvider : SimpleProvider()
pub struct StoriesTestProvider;

impl SimpleProvider for StoriesTestProvider {
    // Translates `override fun arguments(): List<Any> = createStories(HpackJsonUtil.storiesForCurrentDraft())`
    fn arguments(&self) -> Result<Vec<Box<dyn Any>>, Box<dyn std::error::Error>> {
        // HpackJsonUtil.storiesForCurrentDraft() returns Vec<String>
        let stories_list = HpackJsonUtil::stories_for_current_draft();
        
        // HpackDecodeTestBase.createStories(interop_tests: &[String]) returns Vec<Box<dyn Any>>
        // Note: create_stories is a method of HpackDecodeTestBase. 
        // Since it's used as a static-like call in Kotlin (via inheritance or companion),
        // we call it via the HpackDecodeTestBase implementation.
        let args = HpackDecodeTestBase::create_stories(&stories_list);
        
        Ok(args)
    }
}