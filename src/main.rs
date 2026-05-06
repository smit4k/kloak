mod fs;
mod generator;

use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "kloak")]
#[command(about = "View your 6 digit OTPs")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new OTP secret
    Add,

    /// Remove a saved OTP secret
    Remove,

    /// Display all saved OTPs
    List,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{} {error}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add) => add_token(),
        Some(Commands::Remove) => remove_token(),
        Some(Commands::List) | None => list_tokens(),
    }
}

fn list_tokens() -> Result<(), String> {
    let tokens = fs::load_tokens().map_err(|e| e.to_string())?;

    if tokens.is_empty() {
        println!("No OTPs saved. Run `kloak add` to add one.");
        return Ok(());
    }

    let now = current_timestamp()?;
    let remaining = seconds_remaining(now);
    let remaining_label = color_remaining(remaining);

    for token in tokens {
        match generator::generate_token_at(&token.secret, now) {
            Ok(code) => println!("{} - {} - {}", token.issuer, code.bold(), remaining_label),
            Err(_) => println!("{} - {}", token.issuer, "invalid secret".red()),
        }
    }

    Ok(())
}

fn add_token() -> Result<(), String> {
    let mut tokens = fs::load_tokens().map_err(|e| e.to_string())?;
    let issuer = read_prompt("Issuer: ").map_err(|e| e.to_string())?;
    let secret = read_prompt("Secret: ").map_err(|e| e.to_string())?;
    let issuer = issuer.trim().to_string();
    let secret = generator::normalize_secret(secret.trim());

    if issuer.is_empty() {
        return Err("issuer cannot be empty".to_string());
    }

    if secret.is_empty() {
        return Err("secret cannot be empty".to_string());
    }

    if tokens
        .iter()
        .any(|token| token.issuer.eq_ignore_ascii_case(&issuer))
    {
        return Err(format!("issuer `{issuer}` already exists"));
    }

    generator::totp_from_secret(&secret).map_err(|_| "secret is not valid Base32".to_string())?;
    tokens.push(generator::Token::new(issuer.clone(), secret));
    fs::save_tokens(&tokens).map_err(|e| e.to_string())?;

    println!("Added {issuer}.");
    Ok(())
}

fn remove_token() -> Result<(), String> {
    let mut tokens = fs::load_tokens().map_err(|e| e.to_string())?;

    if tokens.is_empty() {
        println!("No OTPs saved.");
        return Ok(());
    }

    for (index, token) in tokens.iter().enumerate() {
        println!("{}. {}", index + 1, token.issuer);
    }

    let selection = read_prompt("Remove number: ").map_err(|e| e.to_string())?;
    let index = selection
        .trim()
        .parse::<usize>()
        .map_err(|_| "enter a valid number".to_string())?;

    if index == 0 {
        return Err("enter a valid number".to_string());
    }

    let removed = remove_token_at(&mut tokens, index - 1)
        .ok_or_else(|| "selection out of range".to_string())?;
    fs::save_tokens(&tokens).map_err(|e| e.to_string())?;

    println!("Removed {}.", removed.issuer);
    Ok(())
}

fn read_prompt(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn remove_token_at(tokens: &mut Vec<generator::Token>, index: usize) -> Option<generator::Token> {
    if index < tokens.len() {
        Some(tokens.remove(index))
    } else {
        None
    }
}

fn current_timestamp() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|e| e.to_string())
}

fn seconds_remaining(timestamp: u64) -> u64 {
    30 - (timestamp % 30)
}

fn color_remaining(remaining: u64) -> ColoredString {
    let label = format!("{remaining}s");

    match remaining {
        16..=30 => label.green(),
        6..=15 => label.yellow(),
        _ => label.red(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_seconds_remaining_in_window() {
        assert_eq!(seconds_remaining(0), 30);
        assert_eq!(seconds_remaining(14), 16);
        assert_eq!(seconds_remaining(15), 15);
        assert_eq!(seconds_remaining(24), 6);
        assert_eq!(seconds_remaining(25), 5);
        assert_eq!(seconds_remaining(29), 1);
    }

    #[test]
    fn removes_selected_token_only() {
        let mut tokens = vec![
            generator::Token::new("First".to_string(), "JBSWY3DPEHPK3PXP".to_string()),
            generator::Token::new("Second".to_string(), "JBSWY3DPEHPK3PXP".to_string()),
        ];

        let removed = remove_token_at(&mut tokens, 0).unwrap();

        assert_eq!(removed.issuer, "First");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].issuer, "Second");
    }

    #[test]
    fn remove_out_of_range_returns_none() {
        let mut tokens = vec![generator::Token::new(
            "Only".to_string(),
            "JBSWY3DPEHPK3PXP".to_string(),
        )];

        assert!(remove_token_at(&mut tokens, 1).is_none());
        assert_eq!(tokens.len(), 1);
    }
}
