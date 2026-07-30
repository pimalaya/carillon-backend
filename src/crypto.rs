//! Password encryption at rest.
//!
//! Carillon holds credentials in order to watch; they must never sit in
//! the database as plaintext. Each password is encrypted to a per-box
//! age (x25519) identity kept in a `0600` key file. Production would
//! move that key into a KMS without changing this interface.

use std::{fs, os::unix::fs::PermissionsExt, path::Path, str::FromStr};

use age::x25519::{Identity, Recipient};
use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use secrecy::zeroize::Zeroizing;
use secrecy::{ExposeSecret, SecretString};

/// Symmetric self-recipient encryptor backed by a persisted age
/// identity.
pub struct Crypto {
    identity: Identity,
    recipient: Recipient,
}

impl Crypto {
    /// Loads an existing age identity from `path`, erroring if the file is
    /// missing. The daemon uses this directly when the store already holds
    /// credentials, so it never silently mints a new key over them; use
    /// [`Crypto::generate`] to create one deliberately.
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("Cannot read age key at {}", path.display()))?;
        let identity =
            Identity::from_str(text.trim()).map_err(|e| anyhow!("Invalid age key: {e}"))?;
        Ok(Self::from_identity(identity))
    }

    /// Generates a fresh age identity and persists it at `path` (mode
    /// `0600`, creating parent directories). Refuses to overwrite an
    /// existing key, so it is safe to invoke deliberately (`keygen`) or on
    /// a genuinely-fresh store.
    pub fn generate(path: &Path) -> Result<Self> {
        if path.exists() {
            bail!(
                "age key already exists at {}; refusing to overwrite",
                path.display()
            );
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create {}", parent.display()))?;
        }
        let identity = Identity::generate();
        let secret = identity.to_string();
        fs::write(path, secret.expose_secret())
            .with_context(|| format!("Cannot write age key at {}", path.display()))?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .context("Cannot restrict age key permissions")?;
        Ok(Self::from_identity(identity))
    }

    /// Builds the encryptor from a loaded identity.
    fn from_identity(identity: Identity) -> Self {
        let recipient = identity.to_public();
        Self {
            identity,
            recipient,
        }
    }

    /// Encrypts a password into a base64 blob for the database.
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let bytes = age::encrypt(&self.recipient, plaintext.as_bytes())
            .context("Cannot encrypt password")?;
        Ok(STANDARD.encode(bytes))
    }

    /// Decrypts a base64 blob produced by [`Crypto::encrypt`].
    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<String> {
        let bytes = STANDARD
            .decode(ciphertext_b64)
            .context("Stored password is not valid base64")?;
        let plaintext = age::decrypt(&self.identity, &bytes).context("Cannot decrypt password")?;
        String::from_utf8(plaintext).context("Decrypted password is not valid UTF-8")
    }

    /// Decrypts a blob straight into a zeroize-on-drop [`SecretString`],
    /// keeping the plaintext out of any long-lived `String`. Preferred over
    /// [`Crypto::decrypt`] on the hot credential path (watcher connect), so
    /// a decrypted secret never lingers past its use (§ hardening,
    /// minimal-residency).
    pub fn decrypt_secret(&self, ciphertext_b64: &str) -> Result<SecretString> {
        let bytes = STANDARD
            .decode(ciphertext_b64)
            .context("Stored secret is not valid base64")?;
        // NOTE: the decrypted bytes zeroize on drop; the SecretString takes
        // its own zeroizing copy, so no plaintext survives this call.
        let plaintext =
            Zeroizing::new(age::decrypt(&self.identity, &bytes).context("Cannot decrypt secret")?);
        let text =
            std::str::from_utf8(&plaintext).context("Decrypted secret is not valid UTF-8")?;
        Ok(SecretString::from(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_crypto() -> Crypto {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "carillon-crypto-test-{}-{n}.key",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let crypto = Crypto::generate(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        crypto
    }

    #[test]
    fn decrypt_secret_round_trips_and_matches_decrypt() {
        let crypto = temp_crypto();
        let enc = crypto.encrypt("hunter2 🔐").unwrap();
        // Both paths recover the same plaintext; the secret one just holds
        // it in a zeroizing type.
        assert_eq!(crypto.decrypt(&enc).unwrap(), "hunter2 🔐");
        assert_eq!(
            crypto.decrypt_secret(&enc).unwrap().expose_secret(),
            "hunter2 🔐"
        );
    }

    #[test]
    fn decrypt_secret_rejects_garbage() {
        let crypto = temp_crypto();
        assert!(crypto.decrypt_secret("not-base64!!").is_err());
    }
}
