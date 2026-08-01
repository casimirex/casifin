//! Benchmarks for amortization schedule generation.

use casifin_sdk::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;

fn bench_fixed_rate_amortization(c: &mut Criterion) {
    let principal = Money::from(200_000);
    let rate = Rate::new(
        Decimal::new(6, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )
    .unwrap();

    c.bench_function("amortization_360_months", |b| {
        b.iter(|| {
            let schedule =
                AmortizationBuilder::new(black_box(principal), black_box(rate), black_box(360))
                    .build()
                    .unwrap();
            black_box(schedule)
        })
    });
}

fn bench_arm_schedule(c: &mut Criterion) {
    let principal = Money::from(300_000);
    let initial_rate = Rate::new(
        Decimal::new(5, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )
    .unwrap();
    let adj_rate = Rate::new(
        Decimal::new(7, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )
    .unwrap();

    c.bench_function("arm_360_months", |b| {
        b.iter(|| {
            let arm = AdjustableRateBuilder::new(
                black_box(principal),
                black_box(initial_rate),
                black_box(360),
            )
            .with_adjustment(61, black_box(adj_rate))
            .build()
            .unwrap();
            black_box(arm)
        })
    });
}

criterion_group!(benches, bench_fixed_rate_amortization, bench_arm_schedule);
criterion_main!(benches);
