//! Benchmarks for amortization schedule generation.

use casifin_sdk::{Casifin, Compounding, Money, Rate};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;

fn amortization_30_year(c: &mut Criterion) {
    let engine = Casifin::with_defaults();
    let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();
    c.bench_function("amortization_30yr", |b| {
        b.iter(|| {
            engine
                .mortgage(
                    black_box(Money::from_decimal(Decimal::new(300000, 0))),
                    black_box(rate),
                    black_box(360),
                )
                .build()
                .unwrap()
        })
    });
}

criterion_group!(benches, amortization_30_year);
criterion_main!(benches);
