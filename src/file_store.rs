use crate::commons::{KeyMaterial, KeyStore, KeyStoreError};
use regex::Regex;
use std::{fs, path::{Path, PathBuf}};

pub struct FileKeyStore {
    path: PathBuf,
}

impl FileKeyStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, KeyStoreError> {
        let path = path.into();
        // if path.exists() {
        //     check_permissions(&path)?;
        // } else if let Some(parent) = path.parent() {
        //     check_permissions(parent)?;
        // }
        Ok(Self { path })
    }
}

impl KeyStore for FileKeyStore {
    fn read_keys(&self) -> Result<Vec<KeyMaterial>, KeyStoreError> {
        let content = std::fs::read_to_string(&self.path)?;
        let regex = Regex::new(r"^([^:]+):([^:]+):(\d+)$").unwrap();
        let mut keys = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(caps) = regex.captures(line) {
                let kid = caps.get(1).unwrap().as_str().to_string();
                let secret = caps.get(2).unwrap().as_str().to_string();
                let created_at_unix = caps.get(3).unwrap().as_str().parse()?;
                keys.push(KeyMaterial { kid, secret, created_at_unix });
            } else {
                return Err(KeyStoreError::InvalidFormat(format!("Invalid line {}: {}", i + 1, line)));
            }
        }

        Ok(keys)
    }

    fn write_keys(&self, keys: &[KeyMaterial]) -> Result<(), KeyStoreError> {
        let mut content = String::new();
        for key in keys {
            content.push_str(&format!("{}:{}:{}\n", key.kid, key.secret, key.created_at_unix));
        }
        std::fs::write(&self.path, content)?;
        // set_secure_file_permissions(&self.path)?;
        Ok(())
    }
}

#[cfg(unix)]
fn check_permissions(path: &Path) -> Result<(), KeyStoreError> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path)?;
    let mode = metadata.permissions().mode();
    if mode & 0o077 != 0 {
        panic!("{} permissions too open (mode {:o})", path.display(), mode);
    }
    Ok(())
}

#[cfg(windows)]
fn check_permissions(_path: &Path) -> Result<(), KeyStoreError> {
    Ok(())
}

fn set_secure_file_permissions(path: &Path) -> Result<(), KeyStoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_and_read_keys() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();

        let store = FileKeyStore::new(&path).unwrap();

        let key = KeyMaterial {
            kid: "test-kid".to_string(),
            secret: "supersecret".to_string(),
            created_at_unix: 1234567890,
        };

        store.write_keys(&[key.clone()]).unwrap();

        let keys = store.read_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].kid, key.kid);
        assert_eq!(keys[0].secret, key.secret);
        assert_eq!(keys[0].created_at_unix, key.created_at_unix);
    }

    #[test]
    fn test_invalid_format_fails() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        let store = FileKeyStore::new(&path).unwrap();

        fs::write(&path, "badly:formatted:line:extra\n").unwrap();

        let result = store.read_keys();
        assert!(result.is_err());
    }
}