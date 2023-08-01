macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("\x1b[31;1m{}\x1b[0m", format_args!($($arg)*))
    };
}

#[cfg(debug_assertions)]
macro_rules! background {
    ($($arg:tt)*) => {
        eprintln!("\x1b[90m{}\x1b[0m", format_args!($($arg)*))
    };
}

macro_rules! data {
    ($($arg:tt)*) => {
        eprintln!("\x1b[33m{}\x1b[0m", format_args!($($arg)*))
    };
}
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!("\x1b[32;1m{}\x1b[0m", format_args!($($arg)*))
    };
}
