use std::sync::Mutex;

use keyring::{Entry, Error as KeyringError};
use rand_core::{OsRng, RngCore};
use serde::Serialize;
use sha2::{Digest, Sha256};
use ts_core::ModelSigningKey as SigningKey;
use zeroize::Zeroizing;

const KEYRING_SERVICE: &str = "com.tensorswarm.desktop";
const KEYRING_ACCOUNT: &str = "publisher-ed25519-v1";
const SIGNING_SEED_BYTES: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SigningIdentityInfo {
    pub exists: bool,
    pub fingerprint: Option<String>,
}

pub(crate) trait SecretBackend: Send + Sync {
    fn get_secret(&self) -> Result<Option<Vec<u8>>, String>;
    fn set_secret(&self, secret: &[u8]) -> Result<(), String>;
}

#[derive(Default)]
pub(crate) struct SigningIdentityManager {
    operation_lock: Mutex<()>,
}

impl SigningIdentityManager {
    pub(crate) fn lock(&self) -> Result<std::sync::MutexGuard<'_, ()>, String> {
        self.operation_lock
            .lock()
            .map_err(|_| "signing identity operation lock is poisoned".to_owned())
    }
}

pub(crate) struct OsCredentialStore;

impl SecretBackend for OsCredentialStore {
    fn get_secret(&self) -> Result<Option<Vec<u8>>, String> {
        let entry = credential_entry()?;
        match entry.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(format!("OS credential store read failed: {error}")),
        }
    }

    fn set_secret(&self, secret: &[u8]) -> Result<(), String> {
        let entry = credential_entry()?;
        entry
            .set_secret(secret)
            .map_err(|error| format!("OS credential store write failed: {error}"))
    }
}

fn credential_entry() -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|error| format!("OS credential store is unavailable: {error}"))
}

pub(crate) fn inspect<B: SecretBackend>(backend: &B) -> Result<SigningIdentityInfo, String> {
    Ok(match load_signing_key(backend)? {
        Some(key) => SigningIdentityInfo {
            exists: true,
            fingerprint: Some(key_fingerprint(&key)),
        },
        None => SigningIdentityInfo {
            exists: false,
            fingerprint: None,
        },
    })
}

pub(crate) fn ensure_identity<B: SecretBackend>(
    backend: &B,
) -> Result<SigningIdentityInfo, String> {
    if let Some(key) = load_signing_key(backend)? {
        return Ok(SigningIdentityInfo {
            exists: true,
            fingerprint: Some(key_fingerprint(&key)),
        });
    }

    let key = generate_signing_key()?;
    let seed = Zeroizing::new(key.to_bytes());
    backend.set_secret(&seed[..])?;

    let stored_key = load_signing_key(backend)?
        .ok_or_else(|| "OS credential store did not retain the signing key".to_owned())?;
    Ok(SigningIdentityInfo {
        exists: true,
        fingerprint: Some(key_fingerprint(&stored_key)),
    })
}

pub(crate) fn with_signing_key<B, T>(
    backend: &B,
    use_key: impl FnOnce(&SigningKey) -> Result<T, String>,
) -> Result<T, String>
where
    B: SecretBackend,
{
    let key = load_signing_key(backend)?
        .ok_or_else(|| "create a local signing identity before signing a release".to_owned())?;
    use_key(&key)
}

fn generate_signing_key() -> Result<SigningKey, String> {
    let mut seed = Zeroizing::new([0_u8; SIGNING_SEED_BYTES]);
    OsRng
        .try_fill_bytes(&mut *seed)
        .map_err(|error| format!("operating-system random generator failed: {error}"))?;
    Ok(SigningKey::from_bytes(&seed))
}

fn load_signing_key<B: SecretBackend>(backend: &B) -> Result<Option<SigningKey>, String> {
    let Some(secret) = backend.get_secret()? else {
        return Ok(None);
    };
    let secret = Zeroizing::new(secret);
    let seed: [u8; SIGNING_SEED_BYTES] = secret
        .as_slice()
        .try_into()
        .map_err(|_| "stored signing key is corrupt; it was not replaced".to_owned())?;
    let seed = Zeroizing::new(seed);
    Ok(Some(SigningKey::from_bytes(&seed)))
}

fn key_fingerprint(key: &SigningKey) -> String {
    let public_key = key.verifying_key().to_bytes();
    fingerprint_public_key(&public_key)
}

pub(crate) fn fingerprint_public_key(public_key: &[u8; 32]) -> String {
    Sha256::digest(public_key)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use ts_core::ModelSigningKey as SigningKey;

    use super::{ensure_identity, inspect, with_signing_key, SecretBackend};

    #[derive(Default)]
    struct MemoryStore {
        secret: Mutex<Option<Vec<u8>>>,
        fail_reads: bool,
    }

    impl SecretBackend for MemoryStore {
        fn get_secret(&self) -> Result<Option<Vec<u8>>, String> {
            if self.fail_reads {
                return Err("test credential-store read failure".to_owned());
            }
            Ok(self.secret.lock().unwrap().clone())
        }

        fn set_secret(&self, secret: &[u8]) -> Result<(), String> {
            *self.secret.lock().unwrap() = Some(secret.to_vec());
            Ok(())
        }
    }

    #[test]
    fn inspection_does_not_create_or_mutate_a_key() {
        let store = MemoryStore::default();
        let status = inspect(&store).unwrap();
        assert!(!status.exists);
        assert_eq!(status.fingerprint, None);
        assert!(store.secret.lock().unwrap().is_none());
    }

    #[test]
    fn key_creation_is_idempotent_and_exposes_only_a_public_fingerprint() {
        let store = MemoryStore::default();
        let first = ensure_identity(&store).unwrap();
        let stored = store.secret.lock().unwrap().clone().unwrap();
        let second = ensure_identity(&store).unwrap();

        assert_eq!(stored.len(), 32);
        assert_eq!(first, second);
        assert!(first.exists);
        assert_eq!(first.fingerprint.as_ref().unwrap().len(), 64);
        assert_ne!(first.fingerprint.as_ref().unwrap().as_bytes(), stored);
    }

    #[test]
    fn invalid_stored_key_fails_closed_without_replacement() {
        let store = MemoryStore {
            secret: Mutex::new(Some(vec![1, 2, 3])),
            fail_reads: false,
        };
        let err = ensure_identity(&store).unwrap_err();
        assert!(err.contains("corrupt"));
        assert_eq!(*store.secret.lock().unwrap(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn key_access_reports_store_failures_without_generating_a_fallback() {
        let store = MemoryStore {
            secret: Mutex::new(None),
            fail_reads: true,
        };
        assert!(inspect(&store).unwrap_err().contains("read failure"));
        assert!(store.secret.lock().unwrap().is_none());
    }

    #[test]
    fn signing_key_is_only_available_inside_the_native_callback() {
        let store = MemoryStore {
            secret: Mutex::new(Some(SigningKey::from_bytes(&[9; 32]).to_bytes().to_vec())),
            fail_reads: false,
        };
        let public_key =
            with_signing_key(&store, |key| Ok(key.verifying_key().to_bytes())).unwrap();
        assert_eq!(
            public_key,
            SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes()
        );
    }
}
