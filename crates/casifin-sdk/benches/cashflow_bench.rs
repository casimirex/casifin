//! Benchmarks for cash flow analysis.

use casifin_sdk::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;

fn bench_npv(c: &mut Criterion) {
    let flows = CashFlowStream::from_vec(vec![
        Money::from(-10_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
    ]);
    let rate = Decimal::new(10, 2);

    c.bench_function("npv_5_periods", |b| {
        b.iter(|| {
            let result = npv(black_box(rate), black_box(&flows)).unwrap();
            black_box(result)
        })
    });
}

fn bench_irr(c: &mut Criterion) {
    let flows = CashFlowStream::from_vec(vec![
        Money::from(-10_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
    ]);

    c.bench_function("irr_5_periods", |b| {
        b.iter(|| {
            let result = irr(
                black_box(&flows),
                Decimal::new(1, 1),
                1000,
                Decimal::new(1, 12),
            )
            .unwrap();
            black_box(result)
        })
    });
}

fn bench_xnpv(c: &mut Criterion) {
    use chrono::NaiveDate;

    let date1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    let date3 = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

    let flows = CashFlowStream::new(vec![
        CashFlow::dated(Money::from(-10_000), date1),
        CashFlow::dated(Money::from(5_000), date2),
        CashFlow::dated(Money::from(5_000), date3),
    ]);
    let rate = Decimal::new(10, 2);

    c.bench_function("xnpv_3_dated_flows", |b| {
        b.iter(|| {
            let result = xnpv(black_box(rate), black_box(&flows)).unwrap();
            black_box(result)
        })
    });
}

criterion_group!(benches, bench_npv, bench_irr, bench_xnpv);
criterion_main!(benches);
