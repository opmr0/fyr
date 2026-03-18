#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{} {}", "[fyr]".red().bold(), format!($($arg)*).red());
    }};
}

#[macro_export]
macro_rules! log {
    ($quiet:expr, $($arg:tt)*) => {{
        if !$quiet {
            use colored::Colorize;
            println!("{} {}", "[fyr]".yellow().bold(), format!($($arg)*));
        }
    }};
}
