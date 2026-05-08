/*
 * Copyright (C) 2020 Square, Inc.
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

use num_bigint::BigInt;
use okio::ByteString;
use std::any::Any;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::BitString::*;

// Assuming these types are defined in the corresponding internal modules
// as per the provided Kotlin source structure.
use crate::okhttp3::tls::internal::der::{
    Adapters, AlgorithmIdentifier, AnyValue, BasicConstraints, BitString,
    Certificate, DerAdapter, DerHeader, DerReader, DerWriter, Extension,
    PrivateKeyInfo, TbsCertificate, Validity, AttributeTypeAndValue,
    SubjectPublicKeyInfo, BasicDerAdapter, ObjectIdentifiers,
};

// ASN.1 adapters adapted from the specifications in [RFC 5280].
pub struct CertificateAdapters;

impl CertificateAdapters {
    // Time ::= CHOICE { utcTime UTCTime, generalTime GeneralizedTime }
    pub fn time() -> Box<dyn DerAdapter<i64>> {
        Box::new(TimeAdapter)
    }

    // Validity ::= SEQUENCE { notBefore Time, notAfter Time }
    pub fn validity() -> BasicDerAdapter<Validity> {
        Adapters::sequence(
            "Validity",
            Self::time_adapter(),
            Self::time_adapter(),
            |v| vec![v.not_before.into(), v.not_after.into()],
            |items| Validity {
                not_before: items[0].clone().downcast::<i64>().unwrap().clone(),
                not_after: items[1].clone().downcast::<i64>().unwrap().clone(),
            },
        )
    }

    // The type of the parameters depends on the algorithm that precedes it.
    fn algorithm_parameters() -> Box<dyn DerAdapter<Option<Box<dyn Any>>>> {
        Adapters::using_type_hint(|type_hint| {
            match type_hint {
                ObjectIdentifiers::SHA256_WITH_RSA_ENCRYPTION => Some(Adapters::NULL),
                ObjectIdentifiers::RSA_ENCRYPTION => Some(Adapters::NULL),
                ObjectIdentifiers::EC_PUBLIC_KEY => Some(Adapters::OBJECT_IDENTIFIER),
                _ => None,
            }
        })
    }

    // AlgorithmIdentifier ::= SEQUENCE { algorithm OBJECT IDENTIFIER, parameters ANY DEFINED BY algorithm OPTIONAL }
    pub fn algorithm_identifier() -> BasicDerAdapter<AlgorithmIdentifier> {
        Adapters::sequence(
            "AlgorithmIdentifier",
            Adapters::OBJECT_IDENTIFIER.as_type_hint(),
            Self::algorithm_parameters(),
            |v| vec![v.algorithm.clone().into(), v.parameters.clone().into()],
            |items| AlgorithmIdentifier {
                algorithm: items[0].clone().downcast::<String>().unwrap().clone(),
                parameters: items[1].clone().downcast::<Option<Box<dyn Any>>>().unwrap().clone(),
            },
        )
    }

    fn basic_constraints() -> BasicDerAdapter<BasicConstraints> {
        Adapters::sequence(
            "BasicConstraints",
            Adapters::BOOLEAN.optional(false),
            Adapters::INTEGER_AS_LONG.optional(),
            |v| vec![v.ca.into(), v.max_intermediate_cas.clone().into()],
            |items| BasicConstraints {
                ca: items[0].clone().downcast::<bool>().unwrap().clone(),
                max_intermediate_cas: items[1].clone().downcast::<Option<i64>>().unwrap().clone(),
            },
        )
    }

    pub fn general_name_dns_name() -> Box<dyn DerAdapter<String>> {
        Adapters::IA5_STRING.with_tag(2)
    }

    pub fn general_name_ip_address() -> Box<dyn DerAdapter<Vec<u8>>> {
        Adapters::OCTET_STRING.with_tag(7)
    }

    pub fn general_name() -> Box<dyn DerAdapter<(Box<dyn Any>, Option<Box<dyn Any>>)>> {
        Adapters::choice(
            Self::general_name_dns_name(),
            Self::general_name_ip_address(),
            Adapters::ANY_VALUE,
        )
    }

    fn subject_alternative_name() -> BasicDerAdapter<Vec<(Box<dyn Any>, Option<Box<dyn Any>>)>> {
        Self::general_name().as_sequence_of()
    }

    fn extension_value() -> BasicDerAdapter<Option<Box<dyn Any>>> {
        Adapters::using_type_hint(|type_hint| {
            match type_hint {
                ObjectIdentifiers::SUBJECT_ALTERNATIVE_NAME => Some(Self::subject_alternative_name()),
                ObjectIdentifiers::BASIC_CONSTRAINTS => Some(Self::basic_constraints()),
                _ => None,
            }
        })
        .with_explicit_box(
            Adapters::OCTET_STRING.tag_class(),
            Adapters::OCTET_STRING.tag(),
            false,
        )
    }

    pub fn extension() -> BasicDerAdapter<Extension> {
        Adapters::sequence(
            "Extension",
            Adapters::OBJECT_IDENTIFIER.as_type_hint(),
            Adapters::BOOLEAN.optional(false),
            Self::extension_value(),
            |v| vec![v.id.clone().into(), v.critical.into(), v.value.clone().into()],
            |items| Extension {
                id: items[0].clone().downcast::<String>().unwrap().clone(),
                critical: items[1].clone().downcast::<bool>().unwrap().clone(),
                value: items[2].clone().downcast::<Option<Box<dyn Any>>>().unwrap().clone(),
            },
        )
    }

    fn attribute_type_and_value() -> BasicDerAdapter<AttributeTypeAndValue> {
        Adapters::sequence(
            "AttributeTypeAndValue",
            Adapters::OBJECT_IDENTIFIER,
            Adapters::any(vec![
                (std::any::TypeId::of::<String>(), Adapters::UTF8_STRING),
                (std::any::TypeId::of::<AnyValue>(), Adapters::ANY_VALUE),
            ]),
            |v| vec![v.attr_type.clone().into(), v.value.clone().into()],
            |items| AttributeTypeAndValue {
                attr_type: items[0].clone().downcast::<String>().unwrap().clone(),
                value: items[1].clone().downcast::<Box<dyn Any>>().unwrap().clone(),
            },
        )
    }

    pub fn rdn_sequence() -> BasicDerAdapter<Vec<Vec<AttributeTypeAndValue>>> {
        Self::attribute_type_and_value().as_set_of().as_sequence_of()
    }

    pub fn name() -> Box<dyn DerAdapter<(Box<dyn Any>, Option<Box<dyn Any>>)>> {
        Adapters::choice(Self::rdn_sequence())
    }

    pub fn subject_public_key_info() -> BasicDerAdapter<SubjectPublicKeyInfo> {
        Adapters::sequence(
            "SubjectPublicKeyInfo",
            Self::algorithm_identifier(),
            Adapters::BIT_STRING,
            |v| vec![v.algorithm.clone().into(), v.subject_public_key.clone().into()],
            |items| SubjectPublicKeyInfo {
                algorithm: items[0].clone().downcast::<AlgorithmIdentifier>().unwrap().clone(),
                subject_public_key: items[1].clone().downcast::<BitString>().unwrap().clone(),
            },
        )
    }

    pub fn tbs_certificate() -> BasicDerAdapter<TbsCertificate> {
        Adapters::sequence(
            "TBSCertificate",
            Adapters::INTEGER_AS_LONG.with_explicit_box(0).optional(0),
            Adapters::INTEGER_AS_BIG_INTEGER,
            Self::algorithm_identifier(),
            Self::name(),
            Self::validity(),
            Self::name(),
            Self::subject_public_key_info(),
            Adapters::BIT_STRING.with_tag(1).optional(),
            Adapters::BIT_STRING.with_tag(2).optional(),
            Self::extension().as_sequence_of().with_explicit_box(3).optional(vec![]),
            |v| vec![
                v.version.into(),
                v.serial_number.clone().into(),
                v.signature.clone().into(),
                (Self::rdn_sequence(), v.issuer.clone()).into(),
                v.validity.clone().into(),
                (Self::rdn_sequence(), v.subject.clone()).into(),
                v.subject_public_key_info.clone().into(),
                v.issuer_unique_id.clone().into(),
                v.subject_unique_id.clone().into(),
                v.extensions.clone().into(),
            ],
            |items| TbsCertificate {
                version: items[0].clone().downcast::<i64>().unwrap().clone(),
                serial_number: items[1].clone().downcast::<BigInt>().unwrap().clone(),
                signature: items[2].clone().downcast::<AlgorithmIdentifier>().unwrap().clone(),
                issuer: items[3].clone().downcast::<(Box<dyn Any>, Vec<Vec<AttributeTypeAndValue>>)>().unwrap().1.clone(),
                validity: items[4].clone().downcast::<Validity>().unwrap().clone(),
                subject: items[5].clone().downcast::<(Box<dyn Any>, Vec<Vec<AttributeTypeAndValue>>)>().unwrap().1.clone(),
                subject_public_key_info: items[6].clone().downcast::<SubjectPublicKeyInfo>().unwrap().clone(),
                issuer_unique_id: items[7].clone().downcast::<Option<BitString>>().unwrap().clone(),
                subject_unique_id: items[8].clone().downcast::<Option<BitString>>().unwrap().clone(),
                extensions: items[9].clone().downcast::<Vec<Extension>>().unwrap().clone(),
            },
        )
    }

    pub fn certificate() -> BasicDerAdapter<Certificate> {
        Adapters::sequence(
            "Certificate",
            Self::tbs_certificate(),
            Self::algorithm_identifier(),
            Adapters::BIT_STRING,
            |v| vec![v.tbs_certificate.clone().into(), v.signature_algorithm.clone().into(), v.signature_value.clone().into()],
            |items| Certificate {
                tbs_certificate: items[0].clone().downcast::<TbsCertificate>().unwrap().clone(),
                signature_algorithm: items[1].clone().downcast::<AlgorithmIdentifier>().unwrap().clone(),
                signature_value: items[2].clone().downcast::<BitString>().unwrap().clone(),
            },
        )
    }

    pub fn private_key_info() -> BasicDerAdapter<PrivateKeyInfo> {
        Adapters::sequence(
            "PrivateKeyInfo",
            Adapters::INTEGER_AS_LONG,
            Self::algorithm_identifier(),
            Adapters::OCTET_STRING,
            |v| vec![v.version.into(), v.algorithm_identifier.clone().into(), v.private_key.clone().into()],
            |items| PrivateKeyInfo {
                version: items[0].clone().downcast::<i64>().unwrap().clone(),
                algorithm_identifier: items[1].clone().downcast::<AlgorithmIdentifier>().unwrap().clone(),
                private_key: items[2].clone().downcast::<ByteString>().unwrap().clone(),
            },
        )
    }

    fn time_adapter() -> Box<dyn DerAdapter<i64>> {
        Self::time()
    }
}

struct TimeAdapter;

impl DerAdapter<i64> for TimeAdapter {
    fn matches(&self, header: DerHeader) -> bool {
        Adapters::UTC_TIME.matches(header) || Adapters::GENERALIZED_TIME.matches(header)
    }

    fn from_der(&self, reader: &mut DerReader) -> i64 {
        let peek_header = reader.peek_header().expect("expected time but was exhausted");

        if peek_header.tag_class == Adapters::UTC_TIME.tag_class() && peek_header.tag == Adapters::UTC_TIME.tag() {
            Adapters::UTC_TIME.from_der(reader)
        } else if peek_header.tag_class == Adapters::GENERALIZED_TIME.tag_class() && peek_header.tag == Adapters::GENERALIZED_TIME.tag() {
            Adapters::GENERALIZED_TIME.from_der(reader)
        } else {
            panic!("expected time but was {} at {}", peek_header, reader);
        }
    }

    fn to_der(&self, writer: &mut DerWriter, value: i64) {
        // [1950-01-01T00:00:00..2050-01-01T00:00:00Z)
        if value >= -631_152_000_000 && value < 2_524_608_000_000 {
            Adapters::UTC_TIME.to_der(writer, value);
        } else {
            Adapters::GENERALIZED_TIME.to_der(writer, value);
        }
    }
}
