use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chrono::NaiveDate;
use shaum::{analyze, RuleContext};

fn bench_analyze(c: &mut Criterion) {
    let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let ctx = RuleContext::default();
    
    c.bench_function("analyze_single_day", |b| b.iter(|| {
        analyze(black_box(date), black_box(&ctx)).unwrap();
    }));

    c.bench_function("analyze_10_years", |b| b.iter(|| {
        let mut d = date;
        for _ in 0..3650 {
            analyze(black_box(d), black_box(&ctx)).unwrap();
            d = d.succ_opt().unwrap();
        }
    }));
}

criterion_group!(benches, bench_analyze);
criterion_main!(benches);
