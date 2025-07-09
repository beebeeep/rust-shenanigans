use std::{collections::HashMap, sync::Arc, time::Instant};

use rand::random;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize)]
struct BucketKey {
    limit: u64,
    bucket: String,
}
#[derive(PartialEq, Serialize, Deserialize)]
struct Counter {
    local: u64,
    remote: u64,
}

#[derive(PartialEq, Serialize, Deserialize)]
struct T {
    buckets: Vec<BucketKey>,
    counters: Vec<Counter>,
}

fn main() {
    let mut t = T {
        buckets: Vec::with_capacity(100),
        counters: Vec::with_capacity(100),
    };
    for i in 0..100 {
        t.buckets.push(BucketKey {
            limit: random(),
            bucket: format!("bucket-{i}"),
        });
        t.counters.push(Counter {
            local: random(),
            remote: random(),
        });
    }

    let start = Instant::now();
    let mut s = flexbuffers::FlexbufferSerializer::new();
    t.serialize(&mut s).unwrap();
    let buf = s.take_buffer();
    println!("serialzation {:?}", start.elapsed());

    let start = Instant::now();
    let d = flexbuffers::Reader::get_root(buf.as_ref()).unwrap();
    let r = T::deserialize(d).unwrap();
    println!("deserialization {:?}", start.elapsed());
    assert_eq!(r.buckets[0].bucket, "bucket-0");
}
