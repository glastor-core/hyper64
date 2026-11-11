use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

use base64::Engine;

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encode");

    for size in [64, 1024, 65536] {
        let input: Vec<u8> = (0..size).map(|i| (i * 31 % 256) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));

        // hyper64 encode
        group.bench_with_input(BenchmarkId::new("hyper64", size), &input, |b, data| {
            b.iter(|| {
                let s = hyper64::encode(black_box(data));
                black_box(s);
            });
        });

        // base64 crate 0.22
        group.bench_with_input(BenchmarkId::new("base64-crate", size), &input, |b, data| {
            b.iter(|| {
                let s = base64::prelude::BASE64_STANDARD.encode(black_box(data));
                black_box(s);
            });
        });
    }

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decode");

    for size in [64, 1024, 65536] {
        let raw: Vec<u8> = (0..size).map(|i| (i * 31 % 256) as u8).collect();
        let encoded = hyper64::encode(&raw);
        group.throughput(Throughput::Bytes(size as u64));

        // hyper64 decode
        group.bench_with_input(BenchmarkId::new("hyper64", size), &encoded, |b, text| {
            b.iter(|| {
                let res = hyper64::decode(black_box(text)).unwrap();
                black_box(res);
            });
        });

        // base64 crate 0.22
        group.bench_with_input(
            BenchmarkId::new("base64-crate", size),
            &encoded,
            |b, text| {
                b.iter(|| {
                    let res = base64::prelude::BASE64_STANDARD
                        .decode(black_box(text))
                        .unwrap();
                    black_box(res);
                });
            },
        );

        // hyper64 in-place zero-alloc decode
        group.bench_with_input(
            BenchmarkId::new("hyper64-in-place", size),
            &encoded,
            |b, text| {
                let mut buf = text.as_bytes().to_vec();
                b.iter(|| {
                    buf.copy_from_slice(text.as_bytes());
                    let res =
                        hyper64::decode_in_place(black_box(&mut buf), &hyper64::STANDARD).unwrap();
                    black_box(res);
                });
            },
        );
    }

    group.finish();
}

fn bench_display_vs_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("DisplayVsAlloc");
    let input: Vec<u8> = (0..1024).map(|i| (i * 37 % 256) as u8).collect();
    group.throughput(Throughput::Bytes(1024));

    // hyper64 zero-alloc display
    group.bench_function("hyper64-display", |b| {
        use core::fmt::Write;
        let mut out = String::with_capacity(2048);
        b.iter(|| {
            out.clear();
            write!(out, "{}", hyper64::display(black_box(&input))).unwrap();
            black_box(&out);
        });
    });

    // base64 crate allocate String
    group.bench_function("base64-crate-alloc", |b| {
        b.iter(|| {
            let s = base64::prelude::BASE64_STANDARD.encode(black_box(&input));
            black_box(s);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode, bench_display_vs_alloc);
criterion_main!(benches);
