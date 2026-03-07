pub mod lock;

use keyring::Entry;

const KEYRING_SERVICE: &str = "kore";
const DB_KEY_ACCOUNT: &str = "db-encryption-key";

/// Retrieve the SQLCipher database encryption key from the OS keychain.
/// If no key exists yet (first launch), a new cryptographically random key
/// is generated, stored in the keychain, and returned.
pub fn get_or_create_db_key() -> Result<String, String> {
    let entry = Entry::new(KEYRING_SERVICE, DB_KEY_ACCOUNT)
        .map_err(|e| format!("Failed to access OS keychain: {}", e))?;

    match entry.get_password() {
        Ok(key) => Ok(key),
        Err(keyring::Error::NoEntry) => {
            let key = generate_secure_key();
            entry
                .set_password(&key)
                .map_err(|e| format!("Failed to store DB key in OS keychain: {}", e))?;
            Ok(key)
        }
        Err(e) => Err(format!("Failed to retrieve DB key from OS keychain: {}", e)),
    }
}

/// Generate a 64-character hex string (32 bytes of entropy) using two UUID v4s.
/// UUID v4 uses the OS CSPRNG internally, making this safe for key generation.
fn generate_secure_key() -> String {
    let part1 = uuid::Uuid::new_v4().simple().to_string(); // 32 hex chars
    let part2 = uuid::Uuid::new_v4().simple().to_string(); // 32 hex chars
    format!("{}{}", part1, part2) // 64 hex chars = 32 bytes
}

/// Rotate the DB encryption key. Stores a new key in the keychain.
/// Note: This does NOT re-encrypt the database — that requires SQLCipher's
/// `PRAGMA rekey` and is handled separately by the DB layer.
pub fn rotate_db_key() -> Result<String, String> {
    let entry = Entry::new(KEYRING_SERVICE, DB_KEY_ACCOUNT)
        .map_err(|e| format!("Failed to access OS keychain: {}", e))?;

    let new_key = generate_secure_key();
    entry
        .set_password(&new_key)
        .map_err(|e| format!("Failed to update DB key in OS keychain: {}", e))?;
    Ok(new_key)
}
