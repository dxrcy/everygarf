pub const RESET: &str = "\x1b[0m";

pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";

pub fn remove_colors(string: &str) -> String {
    let mut out = String::new();
    let mut is_ansi_code = false;
    for ch in string.chars() {
        if is_ansi_code {
            if ch == 'm' {
                is_ansi_code = false;
            }
            continue;
        }
        if ch == '\x1b' {
            is_ansi_code = true;
            continue;
        }
        out.push(ch);
    }
    out
}
