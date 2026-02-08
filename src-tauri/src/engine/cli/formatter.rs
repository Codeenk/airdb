//! CLI Output Formatting Module
//! Provides consistent, colorized output for terminal UX

use colored::Colorize;

pub struct CliFormatter;

impl CliFormatter {
    /// Print a success message
    pub fn success(message: &str) {
        println!("{} {}", "✓".green().bold(), message);
    }

    /// Print an error message
    pub fn error(message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message);
    }

    /// Print a warning message
    pub fn warning(message: &str) {
        println!("{} {}", "⚠".yellow().bold(), message);
    }

    /// Print an info message
    pub fn info(message: &str) {
        println!("{} {}", "ℹ".blue().bold(), message);
    }

    /// Print a section header
    pub fn header(title: &str) {
        println!("\n{}", title.bright_cyan().bold());
        println!("{}", "─".repeat(title.len()).bright_black());
    }

    /// Print a key-value pair
    pub fn kv(key: &str, value: &str) {
        println!("  {}: {}", key.bright_white().bold(), value);
    }

    /// Print a list item
    pub fn item(text: &str) {
        println!("  {} {}", "•".bright_black(), text);
    }

    /// Print a numbered item
    pub fn numbered_item(num: usize, text: &str) {
        println!("  {}. {}", num.to_string().bright_white().bold(), text);
    }

    /// Print a progress indicator
    pub fn progress(current: usize, total: usize, message: &str) {
        let percentage = (current as f32 / total as f32 * 100.0) as u32;
        println!(
            "  {} [{}/{}] {}",
            "▶".bright_blue(),
            current.to_string().bright_white().bold(),
            total,
            message
        );
        print!("  [");
        let filled = percentage / 5;
        for i in 0..20 {
            if i < filled {
                print!("{}", "█".bright_green());
            } else {
                print!("{}", "░".bright_black());
            }
        }
        println!("] {}%", percentage);
    }

    /// Print a table header
    pub fn table_header(columns: &[&str]) {
        let header = columns
            .iter()
            .map(|c| c.bright_white().bold().to_string())
            .collect::<Vec<_>>()
            .join(" │ ");
        println!("  {}", header);
        println!("  {}", "─".repeat(header.len()).bright_black());
    }

    /// Print a table row
    pub fn table_row(values: &[&str]) {
        println!("  {}", values.join(" │ "));
    }

    /// Print a code block
    pub fn code_block(code: &str, language: &str) {
        println!("\n{}", format!("```{}", language).bright_black());
        for line in code.lines() {
            println!("  {}", line.bright_white());
        }
        println!("{}\n", "```".bright_black());
    }

    /// Print a divider
    pub fn divider() {
        println!("{}", "─".repeat(60).bright_black());
    }

    /// Print an empty line
    pub fn blank() {
        println!();
    }

    /// Print a spinner (for long operations)
    pub fn spinner(message: &str) {
        print!("\r{} {}...", "⠋".bright_blue(), message);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
}

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Format duration in human-readable format
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500.00 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }
}
