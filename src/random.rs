use std::sync::atomic::{AtomicU64, Ordering};
static INDEX: AtomicU64 = AtomicU64::new(0);
pub fn random() -> u64 {
<<<<<<< Updated upstream
    //let mut val = INDEX.load(Ordering::Relaxed);
    let mut val = INDEX.fetch_add(1, Ordering::Relaxed);
=======
    let mut val = INDEX.load(Ordering::Relaxed);
    //let mut val = INDEX.fetch_add(1, Ordering::Relaxed);
>>>>>>> Stashed changes
    let prev = val;
    val = val.wrapping_sub(5);
    val = val.wrapping_mul(3);
    let mut val_split: (u32, u32) = unsafe { std::mem::transmute(val) };
    let prev_split: (u32, u32) = unsafe { std::mem::transmute(prev) };
    val_split.0 ^= prev_split.1;
    val_split.1 ^= prev_split.0;
    val = unsafe { std::mem::transmute(val_split) };
    val = byte_shuffle(val);
    val = val.rotate_left(11);
<<<<<<< Updated upstream
    //INDEX.store(val, Ordering::Relaxed);
=======
    INDEX.store(val, Ordering::Relaxed);
>>>>>>> Stashed changes
    val
}
#[inline(always)]
fn byte_shuffle(mut val: u64) -> u64 {
    let mut le_bytes = val.to_le_bytes();

    val = u64::from_be_bytes(le_bytes); // Out of order
    let mut split: (u16, u32, u16) = unsafe { std::mem::transmute(val) };
    split.0 = split.0.rotate_right(3);
    split.1 = split.1.rotate_left(5);
    split.2 = split.2.rotate_right(7);
    val = unsafe { std::mem::transmute(split) };

    le_bytes = val.to_be_bytes();
    val = u64::from_le_bytes(le_bytes);
    val
}
pub fn initialize() {
    let mut start = std::process::id() as u64;
    start = start.wrapping_add(
        (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            & ((!0_u64) as u128)) as u64,
    );
    let temp: u8 = 0;
    start = start.wrapping_mul(((&temp) as *const u8).addr() as u64);
<<<<<<< Updated upstream
    start = start.wrapping_sub(std::env::current_exe().unwrap().into_os_string().len() as u64);
=======
    start = start.wrapping_sub(
        std::env::current_exe()
            .unwrap()
            .into_os_string()
            .len()
            .rotate_left(32) as u64,
    );
>>>>>>> Stashed changes
    println!("Starting at: {start}");
    INDEX.store(start, Ordering::Relaxed);
}
