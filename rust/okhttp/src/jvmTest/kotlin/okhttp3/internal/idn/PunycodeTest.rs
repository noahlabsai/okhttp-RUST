/*
 * Copyright (C) 2022 Square, Inc.
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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::Punycode;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::Punycode::*;

pub struct PunycodeTest;

impl PunycodeTest {
    // https://datatracker.ietf.org/doc/html/rfc3492#section-7.1
    pub fn rfc3492_samples(&self) {
        // (A) Arabic (Egyptian)
        self.test_encode_decode(
            "\u{0644}\u{064a}\u{0647}\u{0645}\u{0627}\u{0628}\u{062a}\u{0643}\u{0644}\u{0645}\u{0648}\u{0634}\u{0639}\u{0631}\u{0628}\u{064a}\u{061f}",
            "xn--egbpdaj6bu4bxfgehfvwxn",
        );

        // (B) Chinese (simplified)
        self.test_encode_decode(
            "\u{4ed6}\u{4eec}\u{4e3a}\u{4ec0}\u{4e48}\u{4e0d}\u{8bf4}\u{4e2d}\u{6587}",
            "xn--ihqwcrb4cv8a8dqg056pqjye",
        );

        // (C) Chinese (traditional)
        self.test_encode_decode(
            "\u{4ed6}\u{5011}\u{7232}\u{4ec0}\u{9ebd}\u{4e0d}\u{8aaa}\u{4e2d}\u{6587}",
            "xn--ihqwctvzc91f659drss3x8bo0yb",
        );

        // (D) Czech
        self.test_encode_decode(
            "Pro\u{010d}prost\u{011b}nemluv\u{00ed}\u{010d}esky",
            "xn--Proprostnemluvesky-uyb24dma41a",
        );

        // (E) Hebrew:
        self.test_encode_decode(
            "\u{05dc}\u{05de}\u{05d4}\u{05d4}\u{05dd}\u{05e4}\u{05e9}\u{05d5}\u{05d8}\u{05dc}\u{05d0}\u{05de}\u{05d3}\u{05d1}\u{05e8}\u{05d9}\u{05dd}\u{05e2}\u{05d1}\u{05e8}\u{05d9}\u{05ea}",
            "xn--4dbcagdahymbxekheh6e0a7fei0b",
        );

        // (F) Hindi (Devanagari)
        self.test_encode_decode(
            "\u{092f}\u{0939}\u{0932}\u{094b}\u{0917}\u{0939}\u{093f}\u{0928}\u{094d}\u{0926}\u{0940}\u{0915}\u{094d}\u{092f}\u{094b}\u{0902}\u{0928}\u{0939}\u{0940}\u{0902}\u{092c}\u{094b}\u{0932}\u{0938}\u{0915}\u{0924}\u{0947}\u{0939}\u{0948}\u{0902}",
            "xn--i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd",
        );

        // (G) Japanese (kanji and hiragana)
        self.test_encode_decode(
            "\u{306a}\u{305c}\u{307f}\u{3093}\u{306a}\u{65e5}\u{672c}\u{8a9e}\u{3092}\u{8a71}\u{3057}\u{3066}\u{304f}\u{308c}\u{306a}\u{3044}\u{306e}\u{304b}",
            "xn--n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa",
        );

        // (H) Korean (Hangul syllables)
        self.test_encode_decode(
            "\u{c138}\u{acc4}\u{c758}\u{baa8}\u{b4e0}\u{c0ac}\u{b78c}\u{b4e4}\u{c774}\u{d55c}\u{ad6d}\u{c5b4}\u{b97c}\u{c774}\u{d574}\u{d55c}\u{b2e4}\u{ba74}\u{c5bc}\u{b9c8}\u{b098}\u{c88b}\u{c744}\u{ae4c}",
            "xn--989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c",
        );

        // (I) Russian (Cyrillic)
        self.test_encode_decode(
            "\u{043f}\u{043e}\u{0447}\u{0435}\u{043c}\u{0443}\u{0436}\u{0435}\u{043e}\u{043d}\u{0438}\u{043d}\u{0435}\u{0433}\u{043e}\u{0432}\u{043e}\u{0440}\u{044f}\u{0442}\u{043f}\u{043e}\u{0440}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}",
            "xn--b1abfaaepdrnnbgefbadotcwatmq2g4l",
        );

        // (J) Spanish
        self.test_encode_decode(
            "Porqu\u{00e9}nopuedensimplementehablarenEspa\u{00f1}ol",
            "xn--PorqunopuedensimplementehablarenEspaol-fmd56a",
        );

        // (K) Vietnamese
        self.test_encode_decode(
            "T\u{1ea1}isaoh\u{1ecd}kh\u{00f4}ngth\u{1ec3}ch\u{1ec9}n\u{00f3}iti\u{1ebf}ngVi\u{1ec7}t",
            "xn--TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g",
        );
    }

    pub fn multiple_labels(&self) {
        self.test_encode_decode(
            "\u{2603}.net",
            "xn--n3h.net",
        );
        self.test_encode_decode(
            "\u{00e5}lg\u{00e5}rd.no",
            "xn--lgrd-poac.no",
        );
        self.test_encode_decode(
            "\u{500b}\u{4eba}.\u{9999}\u{6e2f}",
            "xn--gmqw5a.xn--j6w193g",
        );
        self.test_encode_decode(
            "\u{0443}\u{043f}\u{0440}.\u{0441}\u{0440}\u{0431}",
            "xn--o1ach.xn--90a3ac",
        );
    }

    pub fn non_basic_code_point_in_prefix(&self) {
        assert!(Punycode::decode("xn--c\u{00e5}t-n3h").is_none());
    }

    pub fn non_basic_code_point_in_insertion_coding(&self) {
        assert!(Punycode::decode("xn--cat-\u{00f1}3h").is_none());
    }

    pub fn unterminated_code_point(&self) {
        assert!(Punycode::decode("xn--cat-n").is_none());
    }

    pub fn overflow_i(&self) {
        assert!(Punycode::decode("xn--99999999").is_none());
    }

    pub fn overflow_max_code_point(&self) {
        assert!(Punycode::decode("xn--a-b.net").is_none());
        assert!(Punycode::decode("xn--a-9b.net").is_none());
        assert_eq!(Some("a\u{055a}.net".to_string()), Punycode::decode("xn--a-99b.net"));
        assert_eq!(Some("a\u{6ea0}.net".to_string()), Punycode::decode("xn--a-999b.net"));
        assert_eq!(Some("a\u{D8E2}\u{DF5C}.net".to_string()), Punycode::decode("xn--a-9999b.net"));
        assert!(Punycode::decode("xn--a-99999b.net").is_none());
    }

    pub fn dash_in_prefix(&self) {
        self.test_encode_decode(
            "klmn\u{00f6}pqrst-uvwxy",
            "xn--klmnpqrst-uvwxy-ctb",
        );
    }

    pub fn uppercase_punycode(&self) {
        self.test_decode_only(
            "\u{0644}\u{064a}\u{0647}\u{0645}\u{0627}\u{0628}\u{062a}\u{0643}\u{0644}\u{0645}\u{0648}\u{0634}\u{0639}\u{0631}\u{0628}\u{064a}\u{061f}",
            "XN--EGBPDAJ6BU4BXFGEHFVWXN",
        );
    }

    pub fn mixed_case_punycode(&self) {
        self.test_decode_only(
            "\u{0644}\u{064a}\u{0647}\u{0645}\u{0627}\u{0628}\u{062a}\u{0643}\u{0644}\u{0645}\u{0648}\u{0634}\u{0639}\u{0631}\u{0628}\u{064a}\u{061f}",
            "Xn--EgBpDaJ6Bu4bXfGeHfVwXn",
        );
    }

    // It's invalid to have a label longer than 63 characters. If that's requested, the encoder may
    // overflow and return null.
    pub fn overflow_encoding_oversized_label(&self) {
        let a1000 = "a".repeat(1000);
        let a1000_max_code_point = format!("{}\u{{DBFF}}\u{{DFFF}}", a1000);
        self.test_encode_decode(
            &a1000_max_code_point,
            &format!("xn--{}-nc89312g", a1000),
        );
        assert!(
            Punycode::encode(&a1000_max_code_point.repeat(2)).is_none(),
        );
    }

    pub fn invalid_punycode(&self) {
        assert!(Punycode::decode("xn--ls8h=").is_none());
    }

    fn test_encode_decode(&self, unicode: &str, punycode: &str) {
        assert_eq!(Some(unicode.to_string()), Punycode::decode(punycode));
        assert_eq!(Some(punycode.to_string()), Punycode::encode(unicode));
    }

    fn test_decode_only(&self, unicode: &str, punycode: &str) {
        assert_eq!(Some(unicode.to_string()), Punycode::decode(punycode));
    }
}
