use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static LOG: Mutex<Option<File>> = Mutex::new(None);

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Sets the path to the logging file, if you try to log before doing this it will panic.
///
/// This accepts anything that can be converted into a path, but let's be honest you are going to
/// put a hard coded [String] in it at the start of main.
///
/// The path is relative to the root of the project, so if you wanted to have it show up in a file
/// in src then you would need to give this "src/yadda_yadda"
///
/// I personally recommend just a file called "log"
///
/// You really shouldn't ever run into this but if you ever try to set it twice then this will
/// return Err
///
/// ```no_run
/// # use abes_nice_things::set_log_path;
/// fn main() {
///     set_log_path("log").unwrap();
///     // ...
/// }
/// ```
pub fn set_log_path<P: AsRef<Path>>(path: P) -> Result<(), ()> {
    LOG_PATH.set(path.as_ref().to_path_buf()).map_err(|_| ())
}

/// Logs a singular line into the log.
///
/// Do not use this.
///
/// Use [log]
///
/// This is only public so that the macro works properly and is a worse version of [log]
pub fn logln(msg: String) {
    let mut log = LOG.lock().unwrap();
    if log.is_none() {
        // no log made yet
        if let Some(path) = LOG_PATH.get() {
            // Proper procedure
            *log = Some(std::fs::File::create(path).unwrap());
        } else {
            // Naughty naughty
            panic!("Attempted to log without setting the log file");
        }
    }
    // By this point we have a log
    writeln!(log.as_mut().unwrap(), "{msg}").unwrap();
}

/// Logs a message into the log file as defined in [set_log_path].
///
/// Every message will get its own line.
///
/// You can give this input in the same format as [println] and [format].
///
/// If you have not defined a log file when you run this, it will panic. So to avoid that, define
/// the log file at the start of main.
///
/// ```no_run
/// # use abes_nice_things::log;
/// # fn main() {
///     log!("Hello! My favorite number is: {}", 5);
/// # }
/// ```
///
/// The first time you run this, it will create or truncate the file at the log path.
/// This is important because rerunning the code will clear the log file from the moment the first
/// log happens.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logln(format!($($arg)*));
    }
}
/// [log] but with red text, yes it does reset back to normal after.
///
/// Also if you want to actually see it as red text then you need to print the log file to the
/// terminal.
///
/// The easiest way to do that is with the "cat" command, look into it.
#[macro_export]
macro_rules! log_red {
    ($($arg:tt)*) => {
        $crate::logln(format!("\x1b[31m{}\x1b[0m", format!($($arg)*)));
    }
}
/// [log] but with green text, yes it does reset back to normal after.
///
/// Also if you want to actually see it as red text then you need to print the log file to the
/// terminal.
///
/// The easiest way to do that is with the "cat" command, look into it.
#[macro_export]
macro_rules! log_green {
    ($($arg:tt)*) => {
        $crate::logln(format!("\x1b[32m{}\x1b[0m", format!($($arg)*)));
    }
}
/// [log] but with yellow text, yes it does reset back to normal after.
///
/// Also if you want to actually see it as red text then you need to print the log file to the
/// terminal.
///
/// The easiest way to do that is with the "cat" command, look into it.
#[macro_export]
macro_rules! log_yellow {
    ($($arg:tt)*) => {
        $crate::logln(format!("\x1b[33m{}\x1b[0m", format!($($arg)*)));
    }
}
/// [log] but with blue text, yes it does reset back to normal after.
///
/// Also if you want to actually see it as red text then you need to print the log file to the
/// terminal.
///
/// The easiest way to do that is with the "cat" command, look into it.
#[macro_export]
macro_rules! log_blue {
    ($($arg:tt)*) => {
        $crate::logln(format!("\x1b[34m{}\x1b[0m", format!($($arg)*)));
    }
}
/// [log] but with purple text, yes it does reset back to normal after.
///
/// Also if you want to actually see it as red text then you need to print the log file to the
/// terminal.
///
/// The easiest way to do that is with the "cat" command, look into it.
#[macro_export]
macro_rules! log_purple {
    ($($arg:tt)*) => {
        $crate::logln(format!("\x1b[35m{}\x1b[0m", format!($($arg)*)));
    }
}
/// [log] but with cyan text, yes it does reset back to normal after.
///
/// Also if you want to actually see it as red text then you need to print the log file to the
/// terminal.
///
/// The easiest way to do that is with the "cat" command, look into it.
#[macro_export]
macro_rules! log_cyan {
    ($($arg:tt)*) => {
        $crate::logln(format!("\x1b[36m{}\x1b[0m", format!($($arg)*)));
    }
}

#[test]
#[ignore]
fn log_test() {
    set_log_path("log_test").unwrap();
    log!("Testing");
    log!("Yippy!");
}
