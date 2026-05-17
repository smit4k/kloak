use crate::generator::Token;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};
#[cfg(any(unix, target_os = "redox"))]
use xdg::BaseDirectories;

const TOKENS_FILE: &str = "tokens.json";

#[derive(Debug, PartialEq, Eq)]
pub struct TokenRecord {
    pub token: Token,
    pub secret: Option<String>,
}

#[derive(serde::Deserialize)]
struct StoredToken {
    issuer: String,
    secret: Option<String>,
}

pub fn save_tokens(tokens: &[Token]) -> Result<(), io::Error> {
    let path = tokens_path()?;
    save_tokens_to_path(tokens, &path)
}

pub fn load_token_records() -> Result<Vec<TokenRecord>, io::Error> {
    let path = tokens_path()?;
    load_token_records_from_path(&path)
}

fn tokens_path() -> Result<PathBuf, io::Error> {
    #[cfg(any(unix, target_os = "redox"))]
    {
        BaseDirectories::with_prefix("kloak")
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
            .place_config_file(TOKENS_FILE)
    }

    #[cfg(windows)]
    {
        let app_data = env::var_os("APPDATA")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "APPDATA is not set"))?;
        Ok(PathBuf::from(app_data).join("kloak").join(TOKENS_FILE))
    }

    #[cfg(not(any(unix, target_os = "redox", windows)))]
    {
        let home = env::var_os("HOME")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("kloak")
            .join(TOKENS_FILE))
    }
}

pub fn save_tokens_to_path(tokens: &[Token], path: &Path) -> Result<(), io::Error> {
    let serialized = serde_json::to_string_pretty(tokens).unwrap();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serialized)?;

    Ok(())
}

pub fn load_token_records_from_path(path: &Path) -> Result<Vec<TokenRecord>, io::Error> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let stored_tokens: Vec<StoredToken> = serde_json::from_reader(reader)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(stored_tokens
        .into_iter()
        .map(|stored| TokenRecord {
            token: Token::new(stored.issuer),
            secret: stored.secret,
        })
        .collect())
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
        let records = load_token_records_from_path(&path).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn save_then_load_round_trips_tokens() {
        let path = temp_tokens_path();
        let tokens = vec![Token::new("Example".to_string())];

        save_tokens_to_path(&tokens, &path).unwrap();
        let loaded: Vec<Token> = load_token_records_from_path(&path)
            .unwrap()
            .into_iter()
            .map(|record| record.token)
            .collect();

        assert_eq!(loaded, tokens);
    }

    #[test]
    fn load_token_records_preserves_legacy_secret_for_migration() {
        let path = temp_tokens_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            &path,
            r#"[{"issuer":"Example","secret":"JBSWY3DPEHPK3PXP"}]"#,
        )
        .unwrap();

        let records = load_token_records_from_path(&path).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].token, Token::new("Example".to_string()));
        assert_eq!(records[0].secret, Some("JBSWY3DPEHPK3PXP".to_string()));
    }

    #[test]
    fn saved_tokens_do_not_include_secret_field() {
        let path = temp_tokens_path();
        let tokens = vec![Token::new("Example".to_string())];

        save_tokens_to_path(&tokens, &path).unwrap();
        let contents = fs::read_to_string(&path).unwrap();

        assert!(contents.contains("issuer"));
        assert!(!contents.contains("secret"));
        assert!(!contents.contains("JBSWY3DPEHPK3PXP"));
    }
}
