use crate::Style;
use crate::numbers::*;
use crate::setter;
use std::io::Write;
use std::sync::Arc;
/// A type for handling progress bar rendering. You give it an initial value, a target, and a
/// visual length and bip bop bam you have a progress bar.
///
/// Of course, it isn't actually that simple, but it mostly is (if you don't customize it much).
///
/// # Simplest possible progress bar:
/// 1. You create a new instance by calling [new](ProgressBar::new)
///     You will need to give it the starting value of what you are tracking, the end value, and
///     the visual length (How long the bar is visually)
/// 2. You call [draw](ProgressBar::draw) to put it on the screen initially.
/// 3. When you want to update the progress bar, you call [set](ProgressBar::set) to update the
///    visuals with the new value.
/// 4. You (optionally) call [clear](ProgressBar::clear) once you are done with it because I think
///    it looks nice for the progress bar to disappear once it is done.
/// ```
/// # use abes_nice_things::ProgressBar;
/// # fn main() {
/// let mut progress_bar: ProgressBar<usize> = ProgressBar::new(0, 100, 50); // Step 1
///                                                          // ^ Initial value
///                                                             // ^^^ Target/end value
///                                                                  // ^^ Visual length
/// progress_bar.draw(); // Step 2
///
/// // Step 3
/// for progress in 0..=100 {
///     progress_bar.set(progress);
/// }
///
/// progress_bar.clear(); // Step 4
/// # }
/// ```
/// You might see that and think it looks good enough, but it was not enough for me. So onto the
/// complicated shit!
///
/// # Complicated shit
/// 1. You can set the [done_style](ProgressBar::done_style) which is used for the part that has been
/// completed.
/// ```text
/// [===========>-------]
///  ^^^^^^^^^^^^ This part
/// ```
///
/// 2. You can set the [waiting_style](ProgressBar::waiting_style) which is used for the part that
///    is not done yet
/// ```text
/// [===========>-------]
///              ^^^^^^^ This part
/// ```
///
/// 3. You can set the [base_style](ProgressBar::base_style) which is just what gets used for
///    these guys
/// ```text
/// [===========>-------]
/// ^                   ^
/// ```
/// 4. You can change the [done_char](ProgressBar::done_char)
/// ```text
/// [===========>-------]
///  ^^^^^^^^^^^ These
/// ```
/// 5. You can change the [header_char](ProgressBar::header_char)
/// ```text
/// [===========>-------]
///             ^ This
/// ```
/// 6. You can change the [waiting_char](ProgressBar::waiting_char)
/// ```text
/// [===========>-------]
///              ^^^^^^^ These
/// ```
/// 7. You can make it show the [percent_done](ProgressBar::percent_done)
/// ```text
/// [===========>-------] // About here
/// ```
/// 8. You can make it show the [amound_done](ProgressBar::amount_done)
/// ```text
/// [===========>-------] // Also around here
/// ```
/// 9. You can make it show the [rate](ProgressBar::rate) which will either be (for now) an
///    absolute rate, or a rate in bytes + prefixes. In the same place as the last few
///
/// 10. You can make it show the [eta](ProgressBar::eta) which does go into days. (in the same
///     spot)
///
/// 11. You can change what time frame is used to calculate the [rate](ProgressBar::rate) and
///     [eta](ProgressBar::eta) by changing the [timer](ProgressBar::timer)
///
/// 12. You know all that stuff that goes after the progress bar? Well you can make it appear on a
///     newline instead with [supplementary_newline)(ProgressBar::supplementary_newline)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressBar<T: UnsignedInteger> {
    current: T,
    target: T,
    visual_len: T,
    percent_done: bool,
    amount_done: bool,
    done_style: Style,
    // [Style] used by the characters in the bar that are not done yet
    waiting_style: Style,
    base_style: Style,
    supplementary_newline: bool,
    done_char: char,
    // Last done character in the bar
    header_char: char,
    waiting_char: char,
    rate: Option<Rate>,
    eta: bool,
    prev: Option<(std::time::SystemTime, T)>,
    timer: Timer,
}
impl<T: UnsignedInteger> ProgressBar<T> {
    /// Creates a new bar with the given initial value, target value, and visual length. This
    /// creates a fairly bare bones progress bar. Specifically it creates a bar with no extra data
    /// and no [styling](crate::Style). It creates a bar that has '=' to show the completed area, '-'
    /// to show the uncompleted are, and '>' to connect them. Causing it to look approximately
    /// like:
    ///
    /// [================>--------]
    ///
    /// If you do not want it to look like that, use the other methods.
    ///
    /// Notably, this does not draw the bar, only creates the [ProgressBar] instance.
    pub const fn new(initial: T, target: T, visual_len: T) -> ProgressBar<T> {
        ProgressBar {
            current: initial,
            target,
            visual_len,
            percent_done: false,
            amount_done: false,
            done_style: Style::new(),
            waiting_style: Style::new(),
            base_style: Style::new(),
            supplementary_newline: false,
            done_char: '=',
            header_char: '>',
            waiting_char: '-',
            rate: None,
            eta: false,
            prev: None,
            timer: Timer::MostRecent,
        }
    }
    setter!(
        percent_done = bool,
        amount_done = bool,
        done_style = Style,
        waiting_style = Style,
        base_style = Style,
        supplementary_newline = bool,
        current = T,
        done_char = char,
        header_char = char,
        waiting_char = char,
        rate = Option<Rate>,
        eta = bool,
        visual_len = T,
        target = T,
        timer = Timer,
    );
    /// Draws the visual bar.
    ///
    /// If the bar is already on the screen, then weird stuff will happen. For that reason, if you
    /// are redrawing and updating the visual bar, you should probably use [ProgressBar::set]
    ///
    /// However, [ProgressBar::set] assumes the bar is already drawn, so I suggest using this to
    /// draw the initial bar, then using [ProgressBar::set] to update it.
    pub fn draw(&mut self) {
        assert!(
            self.current <= self.target,
            "Attempted to render a progress bar with a current \
        progress greater than the target ({} vs {})",
            self.current,
            self.target
        );
        let num_done = (self.current * self.visual_len) / self.target;
        print!("\x1b[s");
        print!(
            "{}[{}{}{}{}{}{}]\x1b[0m",
            self.base_style,
            self.done_style,
            self.done_char
                .to_string()
                .as_str()
                .repeat(<T as PrimAs<usize>>::prim_as(num_done).max(1) - 1),
            {
                if <T as PrimAs<usize>>::prim_as(num_done) == 0 {
                    "".to_string()
                } else {
                    self.header_char.to_string()
                }
            },
            self.waiting_style,
            self.waiting_char
                .to_string()
                .as_str()
                .repeat((self.visual_len - num_done).try_into().unwrap()),
            self.base_style
        );
        if self.supplementary_newline {
            println!();
        }
        if self.amount_done {
            print!(" {}/{}", self.current, self.target);
        }
        if self.percent_done {
            print!(
                " {:.2}%",
                (<T as PrimAs<f64>>::prim_as(self.current)
                    / <T as PrimAs<f64>>::prim_as(self.target))
                    * 100.0
            );
        }
        let now = std::time::SystemTime::now();
        if let Some((prev_time, prev_val)) = self.prev {
            assert!(prev_val <= self.current);
            let value_per_second = <T as PrimAs<f64>>::prim_as(self.current - prev_val)
                / (now.duration_since(prev_time).unwrap().as_secs_f64());
            if let Some(rate) = self.rate {
                match rate {
                    Rate::Absolute => {
                        print!(" {value_per_second:.2}/s");
                    }
                    Rate::Bytes => {
                        let (divisor, prefix) = if value_per_second >= 1000000000.0 {
                            // GigaBytes
                            (1000000000.0, 'G')
                        } else if value_per_second >= 1000000.0 {
                            // MegaBytes
                            (1000000.0, 'M')
                        } else if value_per_second >= 1000.0 {
                            // KiloBytes
                            (1000.0, 'K')
                        } else {
                            (1.0, ' ')
                        };
                        print!(" {}{prefix}B", value_per_second / divisor);
                    }
                }
            }
            if self.eta {
                let mut seconds = (<T as PrimAs<f64>>::prim_as(self.target - self.current)
                    / value_per_second) as u64;
                let days = seconds / 86400;
                seconds %= 86400;
                let hours = seconds / 3600;
                seconds %= 3600;
                if days >= 1 {
                    print!(" eta: {days} days, {hours} hours, {seconds} seconds");
                } else if hours >= 1 {
                    print!(" eta {hours} hours, {seconds} seconds");
                } else {
                    print!(" eta: {seconds} seconds");
                }
            }
        } else if let Timer::Mean = self.timer {
            self.prev = Some((now, self.current));
        }
        if let Timer::MostRecent = self.timer {
            self.prev = Some((now, self.current));
        }
        std::io::stdout().flush().unwrap();
    }
    /// Clears the previously drawn progress bar so that it can be drawn again. If this is run
    /// before the progress bar is initially drawn, then it will remove text from the
    /// terminal. Generally it is better to use [set](ProgressBar::set) because it will clear,
    /// update, and redraw the progress bar, but this is useful to remove a completed bar.
    /// ```
    /// # use abes_nice_things::ProgressBar;
    /// # fn main() {
    /// let mut bar = ProgressBar::new(0_usize, 100, 50);
    /// // Initially draw the bar
    /// bar.draw();
    ///
    /// // Do things with the bar
    ///
    /// // Clean up
    /// bar.clear();
    /// # }
    /// ```
    pub fn clear(&self) {
        if self.supplementary_newline {
            print!("\r\x1b[2K\x1b[A\x1b[2K");
        } else {
            print!("\r\x1b[2K");
        }
    }
    /// [clear](ProgressBar::clear), [update](ProgressBar::current), and
    /// [redraw](ProgressBar::draw) the progress bar all in one.
    ///
    /// Just like [clear](ProgressBar::clear), it will cause problems if the bar is not on the
    /// screen when this is called, so call [draw](ProgressBar::draw) before the first time you
    /// call this.
    /// ```
    /// # use abes_nice_things::ProgressBar;
    /// # fn main() {
    /// let mut progress_bar: ProgressBar<usize> = ProgressBar::new(0, 100, 50);
    /// progress_bar.draw();
    /// for progress in 0..=100 {
    ///     progress_bar.set(progress);
    /// }
    /// // Technically optional, but I think it is nice to have the bar disappear once it finishes
    /// progress_bar.clear();
    /// # }
    /// ```
    pub fn set(&mut self, new_val: T) {
        self.clear();
        self.current = new_val;
        self.draw();
    }
    /// Sets up the progress bar to automatically update the visuals and gives back a [Proxy] which
    /// is used to enable that.
    /// ```
    /// # use abes_nice_things::ProgressBar;
    /// # fn main() {
    /// let mut progress_bar = ProgressBar::new(0_usize, 100, 50);
    /// progress_bar.draw();
    /// let mut proxy = progress_bar.auto_update(std::time::Duration::from_millis(500));
    /// for progress in 0..=100 {
    ///     proxy.set(progress); // Changes what will be displayed next update
    /// }
    /// proxy.finish().unwrap().clear(); // Optionally make the bar disappear once finished
    /// # }
    /// ```
    /// This does take ownership of the [ProgressBar] to ensure safety, but you can get the
    /// [ProgressBar] back with [Proxy::finish]
    pub fn auto_update(mut self, interval: std::time::Duration) -> Proxy<T>
    where
        T: HasAtomic,
    {
        let arc = Arc::new(T::Atomic::from(self.current));
        let weak = Arc::downgrade(&arc);
        let handle = std::thread::spawn(move || {
            while let Some(progress) = weak.upgrade() {
                self.set(progress.load(std::sync::atomic::Ordering::Relaxed));
                std::thread::sleep(interval);
            }
            self
        });
        Proxy { arc, handle }
    }
}
/// The enum used to specify how the rate should be shown for a [ProgressBar]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rate {
    /// Rate shown as the float calculated for rate with two decimal places.
    Absolute,
    /// Rate shown as bytes with the correct suffixes up to gigabytes per second.
    Bytes,
}
/// This is used as a remote interface to a [ProgressBar] that is automatically updating. You
/// should not be creating this yourself. It can be used in a similar way to the [ProgressBar]
/// itself, but with reduced functionality. You may not be able to manually clear or draw the bar,
/// but you can still update the value shown.
/// ```
/// # use abes_nice_things::ProgressBar;
/// # use std::time::Duration;
/// # fn main() {
/// let mut progress_bar = ProgressBar::new(0_usize, 100, 50);
/// let mut proxy = progress_bar.auto_update(Duration::from_secs(1));
/// for progress in 0..=100 {
///     proxy.set(progress);
/// }
/// // If you want to have the bar disappear once it is done then you have to do that
/// proxy.finish().unwrap().clear();
/// # }
/// ```
pub struct Proxy<T: UnsignedInteger + HasAtomic> {
    arc: Arc<T::Atomic>,
    handle: std::thread::JoinHandle<ProgressBar<T>>,
}
impl<T: UnsignedInteger + HasAtomic> Proxy<T> {
    /// This changes what value will be displayed by the bar, but will not cause it to visually
    /// update. The bar will always update its visuals every time interval.
    ///
    /// This is roughly equivalent to [ProgressBar::set] and can be used in much the same way. When
    /// you want to change what value will be displayed, use this.
    pub fn set(&self, current: T) {
        self.arc
            .store(current, std::sync::atomic::Ordering::Relaxed);
    }
    /// This will stop the thread that updates the visuals and return the original [ProgressBar]
    /// once it finishes the current visual tick. This is the only way to get the [ProgressBar]
    /// back after using [ProgressBar::auto_update]
    pub fn finish(self) -> Result<ProgressBar<T>, Box<dyn std::any::Any + Send + 'static>> {
        // Dropping the arc tells the thread to stop
        std::mem::drop(self.arc);
        self.handle.join()
    }
}
/// The enum used to specify what range of values to use for calculating the eta and rate for a
/// [ProgressBar]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timer {
    /// Uses the value at the most recent draw to calculate the rate of progress and eta
    MostRecent,
    /// Uses the first value (creating effectively a mean) to calculate the rate of progress and
    /// eta
    Mean,
}
