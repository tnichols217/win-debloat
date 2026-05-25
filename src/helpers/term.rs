use colored::Colorize;

pub fn section(msg: &str) {
    println!("\n{}", format!("── {} ──", msg).cyan().bold());
}

pub fn ok(msg: &str) {
    println!("  {} {}", "[OK]".green().bold(), msg);
}

pub fn info(msg: &str) {
    println!("  {} {}", "[INFO]".blue(), msg);
}

pub fn warn(msg: &str) {
    println!("  {} {}", "[WARN]".yellow(), msg);
}

pub fn err(msg: &str) {
    println!("  {} {}", "[ERR]".bright_red().bold(), msg);
}

pub fn log(caller: &str, msg: &str) {
    println!("  {} {}> {}", "[LOG]".green(), caller.green().bold(), msg);
}

pub fn running(caller: &str, msg: &str) {
    println!(
        "  {} {}> {}",
        "[RUNNING]".bright_green(),
        caller.green().bold(),
        msg
    );
}

pub fn banner() {
    println!(
        "{}",
        r#"
  __      ___       ___      _    _           _
  \ \    / (_)_ __ |   \ ___| |__| |___  __ _| |_
   \ \/\/ /| | '  \| |) / -_) '_ \ / _ \/ _` |  _|
    \_/\_/ |_|_||_||___/\___|_.__/_\___/\__,_|\__|
                   Windows ISO Debloater for Linux
"#
        .cyan()
        .bold()
    );
}
