use abes_nice_things::{
    progress_bar::Timer,
    random::{initialize, random, raw_random},
    ProgressBar, Style,
};
use std::io::{BufRead, Read, Write};
use std::{io::Seek, sync::atomic::*};
const BITS: usize = 4;
const ITERATIONS: u64 = u32::MAX as u64;
const BATCH_SIZE: usize = 1000000;
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
    if let Some((duplicate, batch)) = repeat_finder() {
        println!("Cycle found in batch {batch}, measuring size...");
        println!("Size: {}", cycle_counter(duplicate).unwrap())
    }
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
fn repeat_finder() -> Option<(u64, u64)> {
    println!("\n\n\nRepeat finder");
    let mut random = abes_nice_things::random::get_initializer();
    println!("Starting at {random}");
    let mut write = std::fs::File::create("list").unwrap();
    let mut read = std::io::BufReader::new(std::fs::File::open("list").unwrap());
    let mut progress_bar = *PROGRESS_BAR
        .clone()
        .amount_done(true)
        .target(ITERATIONS / BATCH_SIZE as u64)
        .rate(Some(abes_nice_things::progress_bar::Rate::Absolute))
        .timer(Timer::Mean);
    progress_bar.draw();
    let proxy = progress_bar.auto_update(std::time::Duration::from_millis(500));
    let mut buf = [0; 8];
    let start = std::time::Instant::now();
    //let mut duplicates = std::collections::HashSet::new();
    for iteration in 0..(ITERATIONS / BATCH_SIZE as u64) {
        let mut generated = std::collections::HashSet::with_capacity(BATCH_SIZE);
        for _ in 0..BATCH_SIZE {
            random = raw_random(random);
            if generated.contains(&random) {
                proxy.finish().unwrap().clear();
                return Some((random, iteration));
                //panic!("FOUND DUPLICATE: {random} after {iteration}");
                //duplicates.insert(random);
            }
            generated.insert(random);
        }
        read.rewind().unwrap();
        read.consume(read.buffer().len());
        loop {
            if let Err(std::io::ErrorKind::UnexpectedEof) =
                read.read_exact(&mut buf).map_err(|x| x.kind())
            {
                break;
            }
            let check = u64::from_ne_bytes(buf);
            if generated.contains(&check) {
                proxy.finish().unwrap().clear();
                return Some((random, iteration));
                //duplicates.insert(check);
                //panic!("FOUND DUPLICATE: {random} after {iteration}");
            }
        }
        for num in generated.iter() {
            write.write_all(&num.to_ne_bytes()).unwrap();
        }
        write.sync_data().unwrap();
        proxy.set(iteration);
    }
    proxy.finish().unwrap().clear();
    let elapsed = start.elapsed();

    /*println!("Repetitions:");
    for (index, duplicate) in duplicates.iter().enumerate() {
        println!("{index}: {duplicate}");
    }*/

    println!("Total time: {}", elapsed.as_secs());
    println!(
        "No repetition found in all {} batches, congrats!",
        ITERATIONS / BATCH_SIZE as u64
    );
    None
}
fn cycle_counter(initial: u64) -> Option<u64> {
    //let initial = abes_nice_things::random::get_initializer();
    let mut random = initial;
    let mut progress_bar = PROGRESS_BAR;
    progress_bar.draw();
    let proxy = progress_bar.auto_update(std::time::Duration::from_millis(500));
    let start = std::time::Instant::now();
    for iteration in 0..ITERATIONS {
        random = raw_random(random);
        if random == initial {
            proxy.finish().unwrap().clear();
            return Some(iteration);
        }
        if iteration % 10000 == 0 {
            proxy.set(iteration);
        }
    }
    let elapsed = start.elapsed();
    proxy.finish().unwrap().clear();
    println!("No cycle found!");
    println!("Time taken: {} seconds", elapsed.as_secs());
    println!(
        "Average time: {} nano seconds",
        elapsed.as_nanos() / ITERATIONS as u128
    );
    None
}
fn repeat_finder_threaded() {
    println!("\n\n\nRepeat finder threaded");
    let mut write = std::fs::File::create("list").unwrap();
    let (writer, writer_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || loop {
        let data: u64 = writer_rx.recv().unwrap();
        write.write_all(&data.to_ne_bytes()).unwrap();
    });
    let mut sections = Vec::new();
    for i in 0_u64..10_u64 {
        sections.push(((ITERATIONS / 11) * i)..((ITERATIONS / 11) * (i + 1)));
    }
    let mut progress_bar = *PROGRESS_BAR.clone().amount_done(true);
    progress_bar.draw();
    let proxy = progress_bar.auto_update(std::time::Duration::from_millis(500));
    let raw_proxy = unsafe { proxy.raw_proxy() };

    let mut threads: Vec<ThreadAsync<Option<u64>>> = sections
        .iter()
        .map(|range| {
            ThreadAsync::new(repeat_finder_helper(
                range.start,
                range.end,
                raw_proxy.clone(),
                writer.clone(),
            ))
        })
        .collect();

    loop {
        let mut done = true;
        for thread in threads.iter_mut() {
            if let Some(output) = thread.poll() {
                done = false;
                if let Some(duplicate) = output {
                    panic!("Found duplicate at {duplicate}");
                }
            }
        }
        if done {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    proxy.finish().unwrap().clear();
}
fn repeat_finder_helper(
    start: u64,
    end: u64,
    proxy: std::sync::Arc<std::sync::atomic::AtomicU64>,
    writer: std::sync::mpsc::Sender<u64>,
) -> std::thread::JoinHandle<Option<u64>> {
    std::thread::spawn(move || {
        let mut read = std::io::BufReader::new(std::fs::File::open("list").unwrap());
        let mut buf = [0_u8; 8];
        for place in start..end {
            let random = raw_random(place);
            read.rewind().unwrap();
            read.consume(read.buffer().len());
            loop {
                if let Err(std::io::ErrorKind::UnexpectedEof) =
                    read.read_exact(&mut buf).map_err(|x| x.kind())
                {
                    break;
                }
                let check = u64::from_ne_bytes(buf);
                if random == check {
                    return Some(place);
                }
            }
            writer.send(random).unwrap();
            proxy.fetch_add(1, Ordering::Relaxed);
        }
        None
    })
}
enum ThreadAsync<T: Send + 'static> {
    Data(T),
    Handle(std::thread::JoinHandle<T>),
}
impl<T: Clone + Send + 'static> ThreadAsync<T> {
    fn new(handle: std::thread::JoinHandle<T>) -> Self {
        Self::Handle(handle)
    }
    fn update(&mut self) {
        if let Self::Handle(handle) = self {
            let mut swap_handle = std::thread::spawn(|| {
                loop {}
                todo!()
            });
            std::mem::swap(handle, &mut swap_handle);
            *self = Self::Data(swap_handle.join().unwrap());
        }
    }
    fn get_if_available(&self) -> Option<&T> {
        match self {
            Self::Data(data) => Some(data),
            Self::Handle(_) => None,
        }
    }
    fn get_mut_if_available(&mut self) -> Option<&mut T> {
        match self {
            Self::Data(data) => Some(data),
            Self::Handle(_) => None,
        }
    }
    fn poll(&mut self) -> Option<&T> {
        self.update();
        self.get_if_available()
    }
    fn poll_mut(&mut self) -> Option<&mut T> {
        self.update();
        self.get_mut_if_available()
    }
}
