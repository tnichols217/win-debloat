/// External program binary locations
macro_rules! get_program {
    ($env:literal, $fallback:literal) => {
        match option_env!($env) {
            Some(p) => p,
            None => $fallback,
        }
    };
}

pub const P7ZIP: &str = get_program!("P7ZIP", "7z");
pub const XORRISO: &str = get_program!("XORRISO", "xorriso");
pub const WIMLIB: &str = get_program!("WIMLIB", "wimlib-imagex");
pub const HIVEXREG: &str = get_program!("HIVEXREG", "hivexregedit");
