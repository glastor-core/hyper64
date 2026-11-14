//! Comparative Benchmark: hyper64 vs base64-simd vs base64
//!
//! Evaluates throughput and latency across standard payload tiers:
//! - 64 B: Micro-payloads, hashes, UUIDs, cryptographic signatures
//! - 1 KB: Web tokens, JWTs, HTTP API headers
//! - 64 KB: Images, embedded assets, TLS certificates
//! - 1 MB: Streaming buffers, data blobs, large-scale media chunks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use base64::Engine;

const SIZES: [usize; 4] = [64, 1024, 65536, 1048576];

fn bench_comparative_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Comparative_Encode");

    for &size in &SIZES {
        let input: Vec<u8> = (0..size).map(|i| (i * 31 % 256) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));

        // 1. hyper64 encode
        group.bench_with_input(BenchmarkId::new("hyper64", size), &input, |b, data| {
            b.iter(|| {
                let s = hyper64::encode(black_box(data));
                black_box(s);
            });
        });

        // 2. base64-simd encode
        group.bench_with_input(BenchmarkId::new("base64-simd", size), &input, |b, data| {
            b.iter(|| {
                let s = base64_simd::STANDARD.encode_to_string(black_box(data));
                black_box(s);
            });
        });

        // 3. base64 crate (standard baseline)
        group.bench_with_input(BenchmarkId::new("base64-crate", size), &input, |b, data| {
            b.iter(|| {
                let s = base64::prelude::BASE64_STANDARD.encode(black_box(data));
                black_box(s);
            });
        });
    }

    group.finish();
}

fn bench_comparative_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Comparative_Decode");

    for &size in &SIZES {
        let raw: Vec<u8> = (0..size).map(|i| (i * 31 % 256) as u8).collect();
        let encoded = hyper64::encode(&raw);
        group.throughput(Throughput::Bytes(size as u64));

        // 1. hyper64 decode (allocating Vec<u8>)
        group.bench_with_input(BenchmarkId::new("hyper64", size), &encoded, |b, text| {
            b.iter(|| {
                let res = hyper64::decode(black_box(text)).unwrap();
                black_box(res);
            });
        });

        // 2. hyper64 in-place zero-allocation decode
        group.bench_with_input(BenchmarkId::new("hyper64-in-place", size), &encoded, |b, text| {
            let mut buf = text.as_bytes().to_vec();
            b.iter(|| {
                buf.copy_from_slice(text.as_bytes());
                let res = hyper64::decode_in_place(black_box(&mut buf), &hyper64::STANDARD).unwrap();
                black_box(res);
            });
        });

        // 3. base64-simd decode
        group.bench_with_input(BenchmarkId::new("base64-simd", size), &encoded, |b, text| {
            b.iter(|| {
                let res = base64_simd::STANDARD.decode_to_vec(black_box(text.as_bytes())).unwrap();
                black_box(res);
            });
        });

        // 4. base64 crate decode
        group.bench_with_input(BenchmarkId::new("base64-crate", size), &encoded, |b, text| {
            b.iter(|| {
                let res = base64::prelude::BASE64_STANDARD.decode(black_box(text)).unwrap();
                black_box(res);
            });
        });
    }

    group.finish();
}

criterion_group!(
    name = comparative_benches;
    config = Criterion::default().sample_size(10);
    targets = bench_comparative_encode, bench_comparative_decode
);
criterion_main!(comparative_benches);
