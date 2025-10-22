use abes_nice_things::{
    //progress_bar::Rate,
    random::{initialize, random},
    ProgressBar,
    Style,
};
use std::sync::atomic::*;
const BITS: usize = 4;
const ITERATIONS: u64 = u32::MAX as u64;
fn main() {
    initialize();

    println!("\n\n\n");

    //printer();

    //control();

    //println!("\n\n\n");

    num_frequency();

    println!("\n\n\n");

    bit_frequency();
}

#[allow(dead_code)]
fn printer() {
    for i in 0..1000 {
        println!("{i}: {:b}", random());
    }
}
#[allow(dead_code)]
fn control() {
    println!("Control (with black box)");
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        core::hint::black_box(random());
    }
    let elapsed = start.elapsed();
    println!("{ITERATIONS} in {} seconds", elapsed.as_secs());
    println!(
        "Average: {} nano seconds",
        elapsed.as_nanos() / ITERATIONS as u128
    );

    println!("Control (without black box)");
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        random();
    }
    let elapsed = start.elapsed();
    println!("{ITERATIONS} in {} seconds", elapsed.as_secs());
    println!(
        "Average: {} nano seconds",
        elapsed.as_nanos() / ITERATIONS as u128
    );
}
fn num_frequency() {
    println!("Value frequency:");
    let mut frequency = [0; const { 1 << BITS }];
    let start = std::time::Instant::now();
    let mut progress_bar = *ProgressBar::new(ITERATIONS, 50)
        .done_style(*Style::new().cyan().intense(true))
        .supplementary_newline(true)
        //.amount_done(true)
        .percent_done(true)
        .waiting_style(*Style::new().red())
        .header_char('>')
        .eta(true);
    //.rate(Some(Rate::Absolute));
    progress_bar.draw();
    let progress = (AtomicU64::new(0), AtomicBool::new(false));
    std::thread::scope(|s| {
        let progress_ref = &progress;
        let handle = s.spawn(move || {
            while !progress_ref.1.load(Ordering::Relaxed) {
                progress_bar.set(progress_ref.0.load(Ordering::Relaxed));
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            progress_bar
        });
        for iteration in 0..ITERATIONS {
            let index = (random() & ((1 << BITS) - 1)) as usize;
            frequency[index] += 1;
            if iteration % 10000 == 0 {
                progress.0.store(iteration, Ordering::Relaxed);
            }
        }
        progress.1.store(true, Ordering::Relaxed);
        progress_bar = handle.join().unwrap();
        progress_bar.clear();
    });
    let elapsed = start.elapsed();
    let sum = frequency.iter().sum::<u64>() as f64;
    assert_eq!(frequency.iter().sum::<u64>(), ITERATIONS);
    println!("Target: {}", ITERATIONS / (1 << BITS));
    println!(
        "Target frequency: {:.2}",
        (1.0 / (frequency.len() as f32)) * 100.0
    );
    for (index, frequency) in frequency.iter().enumerate() {
        println!(
            "Value: {index} appeared {frequency} times ({:.2}%).",
            (*frequency as f64 / sum) * 100.0
        );
    }
    let mut furthest: f64 = 0.0;
    let mut furthest_index = 0;
    let len = frequency.len();
    for (index, frequency) in frequency.iter().enumerate() {
        let distance = ((1.0_f64 / (len as f64)) - (*frequency as f64 / sum)).abs() * 100.0;
        if distance > furthest {
            furthest = distance;
            furthest_index = index;
        }
    }
    println!("Furthest distance: {:.2}% ({furthest_index})", furthest);
    println!("\nTotal calc time: {} seconds", elapsed.as_secs());
    println!(
        "Average calc time: {} nano seconds",
        (elapsed / u32::MAX).as_nanos()
    );
}
fn bit_frequency() {
    println!("Bit frequency:");
    let mut frequency: [u64; 64] = [0; 64];
    let start = std::time::Instant::now();
    let mut progress_bar = *ProgressBar::new(ITERATIONS, 50)
        .done_style(*Style::new().cyan().intense(true))
        .supplementary_newline(true)
        //.amount_done(true)
        .percent_done(true)
        .waiting_style(*Style::new().red())
        .header_char('>')
        .eta(true);
    //.rate(Some(Rate::Absolute));
    progress_bar.draw();
    let progress = (AtomicU64::new(0), AtomicBool::new(false));
    std::thread::scope(|s| {
        let progress_ref = &progress;
        let handle = s.spawn(move || {
            while !progress_ref.1.load(Ordering::Relaxed) {
                progress_bar.set(progress_ref.0.load(Ordering::Relaxed));
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            progress_bar
        });
        for iteration in 0..ITERATIONS {
            let num = random();
            for index in 0..64 {
                if (num & (1 << index)) != 0 {
                    frequency[index] += 1;
                }
            }
            if iteration % 1000 == 0 {
                progress.0.store(iteration, Ordering::Relaxed);
            }
        }
        progress.1.store(true, Ordering::Relaxed);
        handle.join().unwrap();
        progress_bar.clear();
    });
    let elapsed = start.elapsed();
    let rel_frequency = frequency.map(|frequency| frequency as f64 / ITERATIONS as f64);
    println!("Target: {}", ITERATIONS / 2);
    println!("Target frequency: 50.00%");

    for (index, frequency) in frequency.iter().enumerate() {
        println!(
            "{index}: {frequency} ({:.2}%)",
            rel_frequency[index] * 100.0
        );
    }
    println!("Total time: {} seconds", elapsed.as_secs());
    println!(
        "Average time: {} nano seconds",
        elapsed.as_nanos() / ITERATIONS as u128
    );
}
