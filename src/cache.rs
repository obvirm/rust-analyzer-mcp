use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct Cache<K, V> {
    store: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
    max_entries: usize,
}

struct CacheEntry<V> {
    value: V,
    created: Instant,
}

impl<K, V> Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_entries,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let store = self.store.read().await;
        let entry = store.get(key)?;

        if entry.created.elapsed() > self.ttl {
            return None;
        }

        Some(entry.value.clone())
    }

    pub async fn set(&self, key: K, value: V) {
        let mut store = self.store.write().await;

        if store.len() >= self.max_entries {
            if let Some(oldest) = store
                .iter()
                .min_by_key(|(_, e)| e.created)
                .map(|(k, _)| k.clone())
            {
                store.remove(&oldest);
            }
        }

        store.insert(
            key,
            CacheEntry {
                value,
                created: Instant::now(),
            },
        );
    }

    pub async fn invalidate(&self, key: &K) {
        self.store.write().await.remove(key);
    }

    pub async fn clear(&self) {
        self.store.write().await.clear();
    }

    pub async fn len(&self) -> usize {
        self.store.read().await.len()
    }

    pub fn is_async(&self) -> bool {
        true
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(300), 1000)
    }
}
