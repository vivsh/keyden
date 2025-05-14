use crate::commons::{KeyManagerConfig, KeyMaterial, KeyStore, KeyStoreError};
use crate::utils::generate_secret;
use indexmap::IndexMap;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct KeyManager {
    inner: RwLock<KeyManagerInner>,
}

pub struct KeyManagerInner {
    store: Arc<dyn KeyStore>,
    last_reload_at: Instant,
    config: KeyManagerConfig,
    keys: IndexMap<String, Arc<KeyMaterial>>,
}

pub struct KeyManagerBuilder {
    store: Arc<dyn KeyStore>,
    size: usize,
    count: usize,
    ttl_secs: u64,
    reload_interval: Duration,
}

impl KeyManagerBuilder {
    fn new(store: Arc<dyn KeyStore>) -> Self {
        Self {
            store,
            size: 128,
            count: 1,
            ttl_secs: 86400,
            reload_interval: Duration::from_secs(30),
        }
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
    pub fn ttl_secs(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }
    pub fn reload_interval(mut self, duration: Duration) -> Self {
        self.reload_interval = duration;
        self
    }

    pub fn build(self) -> Result<KeyManager, KeyStoreError> {
        let keys_list = self.store.read_keys()?;
        let mut keys = IndexMap::new();
        for k in keys_list {
            keys.insert(k.kid.clone(), Arc::new(k));
        }

        Ok(KeyManager {
            inner: RwLock::new(KeyManagerInner {
                store: self.store,
                last_reload_at: Instant::now(),
                config: KeyManagerConfig {
                    size: self.size,
                    count: self.count,
                    ttl_secs: self.ttl_secs,
                    reload_interval: self.reload_interval,
                },
                keys,
            }),
        })
    }
}

impl KeyManager {
    pub fn builder<S: KeyStore>(store: S) -> KeyManagerBuilder {
        KeyManagerBuilder::new(Arc::new(store))
    }

    pub fn get_key(&self, kid: &str) -> Option<Arc<KeyMaterial>> {
        let inner = self.inner.read();
        inner.keys.get(kid).map(Arc::clone)
    }

    pub fn get_current_key(&self) -> Option<Arc<KeyMaterial>> {
        let inner = self.inner.read();
        inner.keys.last().map(|v| Arc::clone(v.1))
    }

    pub fn can_reload(&self) -> bool {
        let inner = self.inner.read();
        inner.last_reload_at.elapsed() >= inner.config.reload_interval
    }

    pub fn reload(&self) -> Result<(), KeyStoreError> {
        let (should_reload, store) = {
            let inner = self.inner.read();
            (inner.last_reload_at.elapsed() >= inner.config.reload_interval, Arc::clone(&inner.store))
        };

        if !should_reload {
            return Ok(());
        }

        let keys_list = store.read_keys()?; // no lock held during file IO

        let mut inner = self.inner.write();
        inner.keys.clear();
        for k in keys_list {
            inner.keys.insert(k.kid.clone(), Arc::new(k));
        }
        inner.last_reload_at = Instant::now();
        Ok(())
    }

    pub fn save_keys(&self) -> Result<(), KeyStoreError> {
        let keys = {
            let inner = self.inner.read();
            inner
                .keys
                .values()
                .map(|k| k.as_ref().clone())
                .collect::<Vec<_>>()
        };
        let inner = self.inner.write();
         inner.store.write_keys(&keys)
    }

    /// CLI: Generate a new key
    /// This will not save the key to the store, you need to call `save_keys` after this.
    pub fn generate_key(&self, kid: String) -> Result<KeyMaterial, KeyStoreError> {
        let (size,) = {
            let inner = self.inner.read();
            (inner.config.size,)
        };

        let secret = generate_secret(size);
        let created_at_unix = time::OffsetDateTime::now_utc().unix_timestamp();
        let key = KeyMaterial {
            kid: kid.clone(),
            secret,
            created_at_unix,
        };

        let mut inner = self.inner.write();
        inner.keys.insert(kid, Arc::new(key.clone()));
        Ok(key)
    }

    /// CLI: Rotate keys if needed
    pub fn rotate_keys(&self) -> Result<bool, KeyStoreError> {
        self.reload()?; // refresh first

        let inner = self.inner.write();
        if inner.keys.len() < inner.config.count {
            let kid = format!("key-{}", time::OffsetDateTime::now_utc().unix_timestamp_nanos());
            self.generate_key(kid)?;
            drop(inner); // drop early
            self.save_keys()?;
            return Ok(true);
        }
        Ok(false)
    }

    /// CLI: Get current key
    pub fn current_key(&self) -> Option<Arc<KeyMaterial>> {
        self.get_current_key()
    }

    /// CLI: List all keys
    pub fn list_keys(&self) -> Vec<Arc<KeyMaterial>> {
        let inner = self.inner.read();
        inner.keys.values().cloned().collect()
    }

    /// CLI: Generate a temporary ad-hoc key (not stored)
    pub fn generate_temp_key(size: usize) -> KeyMaterial {
        let secret = generate_secret(size);
        let created_at_unix = time::OffsetDateTime::now_utc().unix_timestamp();
        KeyMaterial {
            kid: format!("temp-{}", created_at_unix),
            secret,
            created_at_unix,
        }
    }
}