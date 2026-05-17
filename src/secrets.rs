use crate::generator::Token;
use keyring_core::{Entry, Error};

const SERVICE: &str = "kloak";

pub fn save_secret(token: &Token, secret: &str) -> Result<(), String> {
    entry(token)?.set_password(secret).map_err(|e| {
        format!(
            "failed to save secret for `{}` to keyring: {e}",
            token.issuer
        )
    })
}

pub fn load_secret(token: &Token) -> Result<String, String> {
    entry(token)?.get_password().map_err(|e| match e {
        Error::NoEntry => format!("secret for `{}` is missing from keyring", token.issuer),
        e => format!(
            "failed to load secret for `{}` from keyring: {e}",
            token.issuer
        ),
    })
}

pub fn delete_secret(token: &Token) -> Result<(), String> {
    match entry(token)?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(format!(
            "failed to delete secret for `{}` from keyring: {e}",
            token.issuer
        )),
    }
}

fn entry(token: &Token) -> Result<Entry, String> {
    keyring::use_native_store(false)
        .map_err(|e| format!("failed to initialize OS keyring: {e}"))?;
    Entry::new(SERVICE, token.keyring_id())
        .map_err(|e| format!("failed to open keyring entry for `{}`: {e}", token.issuer))
}
