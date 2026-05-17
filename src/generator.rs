use data_encoding::BASE32;
use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, TOTP};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub issuer: String,
}

impl Token {
    pub fn new(issuer: String) -> Self {
        Self { issuer }
    }

    pub fn keyring_id(&self) -> &str {
        &self.issuer
    }
}

pub fn generate_token_at(secret: &str, timestamp: u64) -> Result<String, String> {
    Ok(totp_from_secret(secret)?.generate(timestamp))
}

pub fn totp_from_secret(secret: &str) -> Result<TOTP, String> {
    let bytes = decode_secret(secret).map_err(|e| e.to_string())?;
    let totp = TOTP::new_unchecked(Algorithm::SHA1, 6, 1, 30, bytes);

    Ok(totp)
}

pub fn normalize_secret(secret: &str) -> String {
    secret
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn decode_secret(secret: &str) -> Result<Vec<u8>, data_encoding::DecodeError> {
    let normalized = normalize_secret(secret);
    BASE32.decode(normalized.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_base32_secrets() {
        assert_eq!(
            normalize_secret(" jbsw-y3dp ehpk 3pxp "),
            "JBSWY3DPEHPK3PXP"
        );
    }

    #[test]
    fn generates_known_totp_length() {
        let token = generate_token_at("JBSWY3DPEHPK3PXP", 0).unwrap();
        assert_eq!(token.len(), 6);
        assert!(token.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn rejects_invalid_base32_secret() {
        assert!(totp_from_secret("not valid!").is_err());
    }
}
