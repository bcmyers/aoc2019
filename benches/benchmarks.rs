use std::fs;
use std::io;
use std::time::Duration;

use aoc2019;
use criterion::{criterion_group, criterion_main, Criterion};

fn target_01(c: &mut Criterion) {
    let day01 = fs::read_to_string("data/01.txt").unwrap();
    c.bench_function("day_01", |b| {
        b.iter(|| {
            let reader = io::BufReader::new(day01.as_bytes());
            aoc2019::day01::run(reader).unwrap();
        })
    });
}

fn target_02(c: &mut Criterion) {
    let day02 = fs::read_to_string("data/02.txt").unwrap();
    c.bench_function("day_02", |b| {
        b.iter(|| {
            let reader = io::BufReader::new(day02.as_bytes());
            aoc2019::day02::run(reader).unwrap();
        })
    });
}

fn target_03(c: &mut Criterion) {
    let day03 = fs::read_to_string("data/03.txt").unwrap();
    c.bench_function("day_03", |b| {
        b.iter(|| {
            let reader = io::BufReader::new(day03.as_bytes());
            aoc2019::day03::run(reader).unwrap();
        })
    });
}

fn target_04(c: &mut Criterion) {
    let day04 = fs::read_to_string("data/04.txt").unwrap();
    c.bench_function("day_04", |b| {
        b.iter(|| {
            let reader = io::BufReader::new(day04.as_bytes());
            aoc2019::day04::run(reader).unwrap();
        })
    });
}

fn target_05(c: &mut Criterion) {
    let day05 = fs::read_to_string("data/05.txt").unwrap();
    c.bench_function("day_05", |b| {
        b.iter(|| {
            let reader = io::BufReader::new(day05.as_bytes());
            aoc2019::day05::run(reader).unwrap();
        })
    });
}

fn target_06(c: &mut Criterion) {
    let day06 = fs::read_to_string("data/06.txt").unwrap();
    c.bench_function("day_06", |b| {
        b.iter(|| {
            let reader = io::BufReader::new(day06.as_bytes());
            aoc2019::day06::run(reader).unwrap();
        })
    });
}

fn target_07(c: &mut Criterion) {
    let day07 = fs::read_to_string("data/07.txt").unwrap();
    c.bench_function("day_07", |b| {
        b.iter(|| {
            let reader = io::BufReader::new(day07.as_bytes());
            aoc2019::day07::run(reader).unwrap();
        })
    });
}

criterion_group! {
    name = group;
    config = Criterion::default().warm_up_time(Duration::from_secs(5));
    targets = target_01, target_02, target_03, target_04, target_05, target_06, target_07
}

criterion_main!(group);
