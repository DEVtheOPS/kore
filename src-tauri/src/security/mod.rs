pub mod lock;

use keyring::Entry;
use rand::RngCore;
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "kore";
const DB_KEY_ACCOUNT: &str = "db-encryption-key";

/// Retrieve the SQLCipher database encryption key from the OS keychain.
/// If no key exists yet (first launch), a new cryptographically random key
/// is generated, stored in the keychain, and returned.
///
/// Returns a `Zeroizing<String>` so the key material is wiped from heap
/// memory as soon as the caller drops it.
pub fn get_or_create_db_key() -> Result<Zeroizing<String>, String> {
    let entry = Entry::new(KEYRING_SERVICE, DB_KEY_ACCOUNT)
        .map_err(|e| format!("Failed to access OS keychain: {}", e))?;

    match entry.get_password() {
        Ok(key) => Ok(Zeroizing::new(key)),
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

/// Generate a 64-character hex string with 256 bits of CSPRNG entropy.
///
/// Uses `rand::thread_rng` (backed by the OS CSPRNG) to fill 32 raw bytes,
/// then hex-encodes them.  Unlike UUID v4, every bit is random — no fixed
/// version or variant nibbles reduce the keyspace.
fn generate_secure_key() -> Zeroizing<String> {
    let mut key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    // Encode to hex before wiping the raw bytes.
    let hex: String = key_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    key_bytes.zeroize();
    Zeroizing::new(hex)
}
