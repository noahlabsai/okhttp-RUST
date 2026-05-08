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

use std::net::IpAddr;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// Helper function to interleave two vectors.
// This mimics the behavior of the `okhttp3.internal.interleave` Kotlin function.
fn interleave<T: Clone>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut a_iter = a.into_iter();
    let mut b_iter = b.into_iter();

    loop {
        match (a_iter.next(), b_iter.next()) {
            (Some(val_a), Some(val_b)) => {
                result.push(val_a);
                result.push(val_b);
            }
            (Some(val_a), None) => {
                result.push(val_a);
                result.extend(a_iter);
                break;
            }
            (None, Some(val_b)) => {
                result.push(val_b);
                result.extend(b_iter);
                break;
            }
            (None, None) => break,
        }
    }
    result
}

/*
 * Implementation of HappyEyeballs Sorting Addresses.
 *
 * The current implementation does not address any of:
 *  - Async DNS split by IP class
 *  - Stateful handling of connectivity results
 *  - The prioritisation of addresses
 *
 * https://datatracker.ietf.org/doc/html/rfc8305#section-4
 */
pub(crate) fn reorder_for_happy_eyeballs(addresses: Vec<IpAddr>) -> Vec<IpAddr> {
    if addresses.len() < 2 {
        return addresses;
    }

    // partition { it is Inet6Address }
    let (ipv6, ipv4): (Vec<IpAddr>, Vec<IpAddr>) = addresses
        .into_iter()
        .partition(|addr| addr.is_ipv6());

    if ipv6.is_empty() || ipv4.is_empty() {
        // If one of the lists is empty, the original order (or the partitioned result) 
        // is effectively the same as the input list in terms of content.
        // To preserve the exact behavior of returning the original 'addresses' list:
        // we combine them back if we didn't interleave.
        let mut combined = ipv6;
        combined.extend(ipv4);
        combined
    } else {
        interleave(ipv6, ipv4)
    }
}