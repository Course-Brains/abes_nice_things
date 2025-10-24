use abes_nice_things::{
    //progress_bar::Rate,
    random::{initialize, random},
    ProgressBar,
    Style,
};
use std::sync::atomic::*;
const BITS: usize = 4;
const ITERATIONS: u64 = u32::MAX as u64;
const PROGRESS_BAR: ProgressBar<u64> = *ProgressBar::new(0, ITERATIONS, 50)
    .done_style(*Style::new().cyan().intense(true))
    .waiting_style(*Style::new().red())
    .header_char('>')
    .supplementary_newline(true)
    .percent_done(true)
    .eta(true);
fn main() {
    initialize();
    //printer();
    control();
    num_frequency();
    byte_frequency();
    bit_total_frequency();
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
    println!("\n\n\nControl (with black box)");
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

    println!("\n\n\nControl (without black box)");
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
    println!("\n\n\nValue frequency:");
    let mut frequency = [0; const { 1 << BITS }];
    let start = std::time::Instant::now();
    let mut progress_bar = PROGRESS_BAR;
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
    println!("\n\n\nBit frequency:");
    let mut frequency: [u64; 64] = [0; 64];
    let start = std::time::Instant::now();
    let mut progress_bar = PROGRESS_BAR;
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
fn byte_frequency() {
    println!("\n\n\nByte frequency:");
    let mut frequency = [0_u64; 256];
    let start = std::time::Instant::now();
    let mut progress_bar = PROGRESS_BAR;
    progress_bar.draw();
    let progress = (AtomicU64::new(0), AtomicBool::new(false));
    let progress_ref = &progress;
    std::thread::scope(|s| {
        let handle = s.spawn(move || {
            while !progress_ref.1.load(Ordering::Relaxed) {
                progress_bar.set(progress_ref.0.load(Ordering::Relaxed));
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            progress_bar
        });
        for iteration in 0..ITERATIONS {
            let random = random();
            for offset in (0..=56).step_by(8) {
                frequency[((random >> offset) & 255) as usize] += 1;
            }
            if iteration % 10000 == 0 {
                progress_ref.0.store(iteration, Ordering::Relaxed);
            }
        }
        progress.1.store(true, Ordering::Relaxed);
        handle.join().unwrap().clear();
    });
    let elapsed = start.elapsed();
    let expected = (ITERATIONS / 256) * 8;
    println!("Expected: {expected}");
    println!("Expected percent: {}%", (1.0 / 256.0) * 100.0);
    let sum: u128 = frequency.map(|freq| freq as u128).iter().sum();
    for (value, frequency) in frequency.iter().enumerate() {
        let percent_of_total = (*frequency as f64 / sum as f64) * 100.0;
        println!("{value}: {frequency} ({percent_of_total:.4}%)");
    }
    println!("Total time: {} seconds", elapsed.as_secs());
    println!(
        "Average time: {} nano seconds",
        elapsed.as_nanos() / ITERATIONS as u128
    );
}
fn bit_total_frequency() {
    println!("\n\n\nBit total frequency:");
    let mut frequency = [0_u64; 64];
    let mut progress_bar = PROGRESS_BAR;
    progress_bar.draw();
    let proxy = progress_bar.auto_update(std::time::Duration::from_millis(500));
    let start = std::time::Instant::now();
    for iteration in 0..ITERATIONS {
        let random = random();
        let mut total = 0;
        for bit in 0..64 {
            if (random & (1 << bit)) != 0 {
                total += 1;
            }
        }
        frequency[total] += 1;
        if iteration % 1000 == 0 {
            proxy.set(iteration);
        }
    }
    let elapsed = start.elapsed();
    proxy.finish().unwrap().clear();
    let sum = frequency.iter().sum::<u64>();
    for (total, frequency) in frequency.iter().enumerate() {
        println!(
            "{total}: {frequency} ({:.2}%)",
            *frequency as f64 / sum as f64 * 100.0
        );
    }
    println!("Total time: {} seconds", elapsed.as_secs());
    println!(
        "Average time: {} nano seconds",
        elapsed.as_nanos() / ITERATIONS as u128
    );
}
