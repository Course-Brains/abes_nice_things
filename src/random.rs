use std::sync::atomic::{AtomicU64, Ordering};
static INDEX: AtomicU64 = AtomicU64::new(0);
/// A psudo random number generator.
///
/// It generates a mostly random [u64]. It does loop, however testing has shown that the size of
/// the loop exluding calls to enter the loop tends to be about 120 million random calls long. If
/// you want a proper random number generator, use rand.
///
/// # Usage
/// ```
/// # use abes_nice_things::random::{random, initialize};
/// # fn main() {
/// // Don't forget to initialize the rng once!
/// initialize();
///
/// println!("Your random number is: {}", random());
/// # }
/// ```
/// # Semantics
/// This uses the output from the previous run as input for the current. Meaning that if two
/// threads get a random call at the same time, they could both get the same random number.
///
/// This loops for the same reason.
pub fn random() -> u64 {
    let random = raw_random(INDEX.load(Ordering::Relaxed));
    INDEX.store(random, Ordering::Relaxed);
    random
}
/// The actual operation done by [random]. It is not recommended to use this unless you are sure as
/// [random] uses a global state which will have actions in other threads affect the state, thus
/// making it more random. However, this can be used to more deterministically get random numbers.
///
/// As said in the documentation for [random], this uses the previous number as input. It will be
/// substantially less random if you give an incrementing number instead, I do not recommend it.
///
/// # Usage
/// ```
/// # use abes_nice_things::random::raw_random;
/// # fn main() {
/// let input = 5723856;
/// println!("{input} goes to {}", raw_random(input));
/// # }
/// ```
pub fn raw_random(mut val: u64) -> u64 {
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
/// It is not recommended to use this, use [initialize] instead.
///
/// Anyway, this is essentially a random number generator that is used to initialize the proper
/// random number generator. However, this relies on external state in such a way that it becomes
/// substantially less random when run in succession. I recommend only running it once per program.
/// But it does just return a random [u64] so if you want to use it to initialize a local state for
/// [raw_random] then you can.
/// ```
/// # use abes_nice_things::random::{get_initializer, raw_random};
/// # fn main() {
/// let mut local_state = get_initializer();
/// for i in 0..4 {
///     let random = raw_random(local_state);
///     local_state = random;
///     println!("random number {i} is {random}");
/// }
/// # }
/// ```
pub fn get_initializer() -> u64 {
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
    start = start.wrapping_sub(
        std::env::current_exe()
            .unwrap()
            .into_os_string()
            .len()
            .rotate_left(32) as u64,
    );
    start
}
/// A function which initializes the global state for [random](crate::random::random).
/// This is just an external state reliant random number generator, however it is not
/// recommended to run this often as it will be notably less random if run often.
///
/// # Usage:
/// ```
/// # use abes_nice_things::random::{random, initialize};
/// # fn main() {
/// initialize();
/// println!("The random number is {}", random());
/// # }
/// ```
/// # Semantics:
/// If this is not run, then the random state will always start at the same value, do with that
/// what you will.
///
/// This does use [get_initializer] to set the state.
pub fn initialize() {
    INDEX.store(get_initializer(), Ordering::Relaxed);
}
