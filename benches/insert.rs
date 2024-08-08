use std::{hash::RandomState, io::BufRead as _};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use hyperbitbit::HyperBitBit;
use hyperloglogplus::{HyperLogLog as _, HyperLogLogPlus};
use hypertwobits::{HyperBitBit64, HyperTwoBits, M1024, M128, M2048, M256, M4096, M512, M64};

fn bench_ulysses(c: &mut Criterion) {
    let path = "data/Ulysses.csv";
    let file = std::fs::File::open(path).unwrap();
    let buf = std::io::BufReader::new(file);
    let lines = buf.lines().collect::<Result<Vec<String>, _>>().unwrap();

    let mut group = c.benchmark_group("Ulysses");

    group.throughput(Throughput::Elements(lines.len() as u64));

    group.bench_with_input("HyperLogLogPlus", &lines, |b, lines| {
        let mut counter: HyperLogLogPlus<[u8], RandomState> =
            HyperLogLogPlus::new(16, RandomState::new()).unwrap();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line.as_bytes());
            }
        });
    });

    group.bench_with_input("HyperBitBit", &lines, |b, lines| {
        let mut counter = HyperBitBit::new();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });

    group.bench_with_input("HBB64", &lines, |b, lines| {
        let mut counter = HyperBitBit64::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line.as_bytes());
            }
        });
    });
    group.bench_with_input("HyperTwoBits<64>", &lines, |b, lines| {
        let mut counter = HyperTwoBits::<M64>::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });
    group.bench_with_input("HyperTwoBits<128>", &lines, |b, lines| {
        let mut counter = HyperTwoBits::<M128>::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });

    group.bench_with_input("HyperTwoBits<265>", &lines, |b, lines| {
        let mut counter: HyperTwoBits<_> = HyperTwoBits::<M256>::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });

    group.bench_with_input("HyperTwoBits<512>", &lines, |b, lines| {
        let mut counter = HyperTwoBits::<M512>::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });
    group.bench_with_input("HyperTwoBits<1024>", &lines, |b, lines| {
        let mut counter = HyperTwoBits::<M1024>::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });
    group.bench_with_input("HyperTwoBits<2048>", &lines, |b, lines| {
        let mut counter = HyperTwoBits::<M2048>::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });
    group.bench_with_input("HyperTwoBits<4096>", &lines, |b, lines| {
        let mut counter = HyperTwoBits::<M4096>::default();
        b.iter(|| {
            for line in lines.iter() {
                counter.insert(line);
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_ulysses);
criterion_main!(benches);
