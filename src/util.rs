use std::io::{self, Write};

#[allow(dead_code)]
pub fn prompt_confirm(message: &str) -> bool {
    print!("{message} [Y/n] ");
    io::stdout().flush().ok();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    let trimmed = input.trim().to_lowercase();
    trimmed.is_empty() || trimmed == "y" || trimmed == "yes"
}
