/*
 * Copyright (C) 2025 Square, Inc.
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

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

/*
 * An immutable collection of key-value pairs implemented as a singly-linked list.
 *
 * Build up a collection by starting with [EmptyTags] and repeatedly calling [plus]. Each such call
 * returns a new instance.
 *
 * This collection is optimized for safe concurrent access over a very small number of elements.
 *
 * This collection and is expected to hold fewer than 10 elements. Each operation is _O(N)_, and so
 * building an instance with _N_ elements is _O(N**2)_.
 */
#[derive(Clone)]
pub enum Tags {
    Empty,
    Linked(Arc<LinkedTags>),
}

impl Default for Tags {
    fn default() -> Self {
        Tags::Empty
    }
}

pub const Linked: Tags = Tags::Linked;

impl Tags {
    /*
     * Returns a tags instance that maps [key] to [value]. If [value] is null, this returns a tags
     * instance that does not have any mapping for [key].
     */
    pub fn plus<T: Any>(&self, key: TypeId, value: Option<Arc<T>>) -> Tags {
        match self {
            Tags::Empty => {
                if let Some(v) = value {
                    Tags::Linked(Arc::new(LinkedTags {
                        key,
                        value: Arc::new(v) as Arc<dyn Any>,
                        next: Tags::Empty,
                    }))
                } else {
                    Tags::Empty
                }
            }
            Tags::Linked(linked) => {
                // Create a copy of this `LinkedTags` that doesn't have a mapping for `key`.
                let this_minus_key = if *linked.key == key {
                    linked.next.clone()
                } else {
                    let next_minus_key = linked.next.plus(key, None);
                    if let Tags::Linked(ref next_linked) = linked.next {
                        if Arc::ptr_eq(next_linked, match &next_minus_key {
                            Tags::Linked(l) => l,
                            _ => panic!("Unexpected EmptyTags"),
                        }) {
                            Tags::Linked(linked.clone())
                        }
                    } else if let Tags::Empty = linked.next {
                        if let Tags::Empty = next_minus_key {
                            Tags::Linked(linked.clone())
                        } else {
                            Tags::Linked(Arc::new(LinkedTags {
                                key: *linked.key,
                                value: linked.value.clone(),
                                next: next_minus_key,
                            }))
                        }
                    } else {
                        Tags::Linked(Arc::new(LinkedTags {
                            key: *linked.key,
                            value: linked.value.clone(),
                            next: next_minus_key,
                        }))
                    }
                };

                // Return a new `Tags` that maps `key` to `value`.
                if let Some(v) = value {
                    Tags::Linked(Arc::new(LinkedTags {
                        key,
                        value: Arc::new(v) as Arc<dyn Any>,
                        next: this_minus_key,
                    }))
                } else {
                    this_minus_key
                }
            }
        }
    }

    pub fn get<T: Any>(&self, key: TypeId) -> Option<Arc<T>> {
        match self {
            Tags::Empty => None,
            Tags::Linked(linked) => {
                if *linked.key == key {
                    linked.value.clone().downcast::<T>().ok()
                } else {
                    linked.next.get(key)
                }
            }
        }
    }
}

impl std::fmt::Debug for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tags::Empty => f.write_str("{}"),
            Tags::Linked(linked) => {
                let mut elements = Vec::new();
                let mut current = Some(linked.clone());
                while let Some(node) = current {
                    elements.push(node);
                    current = match &node.next {
                        Tags::Linked(n) => Some(n.clone()),
                        Tags::Empty => None,
                    };
                }
                elements.reverse();
                let formatted: Vec<String> = elements
                    .into_iter()
                    .map(|node| format!("{:?}={:?}", node.key, node.value))
                    .collect();
                write!(f, "{{{}}}", formatted.join(","))
            }
        }
    }
}

struct LinkedTags {
    key: TypeId,
    value: Arc<dyn Any>,
    next: Tags,
}

/*
 * Rust implementation of the AtomicReference.computeIfAbsent logic.
 * Since Rust doesn't have a direct equivalent to AtomicReference<Tags> that allows 
 * easy CAS on an enum, we use a Mutex or a similar synchronization primitive.
 * However, to preserve the "lock-free" spirit of the original Kotlin code, 
 * we use an AtomicPtr or a Mutex-wrapped Arc.
 */
pub fn compute_if_absent<T: Any, F>(
    atomic_tags: &std::sync::Mutex<Tags>,
    type_id: TypeId,
    compute: F,
) -> Arc<T>
where
    F: Fn() -> Arc<T>,
{
    loop {
        let tags = atomic_tags.lock().unwrap().clone();

        // If the element is already present. Return it.
        if let Some(existing) = tags.get::<T>(type_id) {
            return existing;
        }

        let computed = compute();

        // If we successfully add the computed element, we're done.
        let new_tags = tags.plus(type_id, Some(computed.clone()));
        
        let mut lock = atomic_tags.lock().unwrap();
        // In a real lock-free scenario, we'd CAS here. With Mutex, we check if it changed.
        if std::ptr::eq(&*lock, &tags) || match (&*lock, &tags) {
            (Tags::Empty, Tags::Empty) => true,
            (Tags::Linked(a), Tags::Linked(b)) => Arc::ptr_eq(a, b),
            _ => false,
        } {
            *lock = new_tags;
            return computed;
        }
        // We lost the race. Try again!
    }
}
)}
