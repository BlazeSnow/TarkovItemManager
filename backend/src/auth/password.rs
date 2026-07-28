use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::TryRngCore;
pub fn hash(password: &str) -> Result<String> {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let salt = SaltString::encode_b64(&bytes).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
pub fn verify(password: &str, hash: &str) -> Result<()> {
    let parsed = PasswordHash::new(hash).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let hash = hash("a-secure-password").unwrap();
        assert!(verify("a-secure-password", &hash).is_ok());
        assert!(verify("wrong-password", &hash).is_err())
    }
}
