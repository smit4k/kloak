use crate::generator::Token;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

const TOKENS_FILE: &str = "tokens.json";

pub fn save_tokens(tokens: &[Token]) -> Result<(), io::Error> {
    let path = tokens_path()?;
    save_tokens_to_path(tokens, &path)
}

pub fn load_tokens() -> Result<Vec<Token>, io::Error> {
    let path = tokens_path()?;
    load_tokens_from_path(&path)
}

fn tokens_path() -> Result<PathBuf, io::Error> {
    BaseDirectories::with_prefix("kloak")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .place_config_file(TOKENS_FILE)
}

pub fn save_tokens_to_path(tokens: &[Token], path: &Path) -> Result<(), io::Error> {
    let serialized = serde_json::to_string_pretty(tokens).unwrap();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serialized)?;

    Ok(())
}

pub fn load_tokens_from_path(path: &Path) -> Result<Vec<Token>, io::Error> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tokens = serde_json::from_reader(reader)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_tokens_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("kloak-test-{timestamp}"));
        path.push(TOKENS_FILE);
        path
    }

    #[test]
    fn missing_file_loads_empty_tokens() {
        let path = temp_tokens_path();
        let tokens = load_tokens_from_path(&path).unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn save_then_load_round_trips_tokens() {
        let path = temp_tokens_path();
        let tokens = vec![Token::new(
            "Example".to_string(),
            "JBSWY3DPEHPK3PXP".to_string(),
        )];

        save_tokens_to_path(&tokens, &path).unwrap();
        let loaded = load_tokens_from_path(&path).unwrap();

        assert_eq!(loaded, tokens);
    }
}
