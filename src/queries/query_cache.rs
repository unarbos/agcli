//! In-memory TTL cache for repeated chain queries.
//!
//! Caches expensive read-only queries (subnet list, dynamic info)
//! with a short TTL (default 30s) to avoid redundant chain calls
//! within the same command session.

use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::types::chain_data::{DynamicInfo, SubnetInfo};

/// Default cache TTL in seconds.
const DEFAULT_TTL_SECS: u64 = 30;

/// Shared query cache for chain data that changes slowly.
#[derive(Clone)]
pub struct QueryCache {
    /// Cached subnet list (all subnets).
    subnets: Cache<(), Arc<Vec<SubnetInfo>>>,
    /// Cached dynamic info for all subnets.
    all_dynamic: Cache<(), Arc<Vec<DynamicInfo>>>,
    /// Cached dynamic info per subnet.
    dynamic_by_netuid: Cache<u16, Arc<DynamicInfo>>,
}

impl QueryCache {
    /// Create a new cache with the default TTL.
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(DEFAULT_TTL_SECS))
    }

    /// Create a cache with a custom TTL.
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            subnets: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1)
                .build(),
            all_dynamic: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1)
                .build(),
            dynamic_by_netuid: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(100)
                .build(),
        }
    }

    /// Get or fetch all subnets.
    pub async fn get_all_subnets<F, Fut>(&self, fetch: F) -> anyhow::Result<Arc<Vec<SubnetInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Vec<SubnetInfo>>>,
    {
        if let Some(cached) = self.subnets.get(&()) .await {
            return Ok(cached);
        }
        let data = Arc::new(fetch().await?);
        self.subnets.insert((), data.clone()).await;
        Ok(data)
    }

    /// Get or fetch all dynamic info.
    pub async fn get_all_dynamic_info<F, Fut>(
        &self,
        fetch: F,
    ) -> anyhow::Result<Arc<Vec<DynamicInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Vec<DynamicInfo>>>,
    {
        if let Some(cached) = self.all_dynamic.get(&()).await {
            return Ok(cached);
        }
        let data = Arc::new(fetch().await?);
        self.all_dynamic.insert((), data.clone()).await;
        // Also populate per-netuid cache
        for d in data.iter() {
            self.dynamic_by_netuid
                .insert(d.netuid.0, Arc::new(d.clone()))
                .await;
        }
        Ok(data)
    }

    /// Get or fetch dynamic info for a specific subnet.
    pub async fn get_dynamic_info<F, Fut>(
        &self,
        netuid: u16,
        fetch: F,
    ) -> anyhow::Result<Option<Arc<DynamicInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Option<DynamicInfo>>>,
    {
        if let Some(cached) = self.dynamic_by_netuid.get(&netuid).await {
            return Ok(Some(cached));
        }
        match fetch().await? {
            Some(data) => {
                let arc = Arc::new(data);
                self.dynamic_by_netuid.insert(netuid, arc.clone()).await;
                Ok(Some(arc))
            }
            None => Ok(None),
        }
    }

    /// Invalidate all cached data.
    pub async fn invalidate_all(&self) {
        self.subnets.invalidate_all();
        self.all_dynamic.invalidate_all();
        self.dynamic_by_netuid.invalidate_all();
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn cache_deduplicates_calls() {
        let cache = QueryCache::new();
        let call_count = Arc::new(AtomicU32::new(0));

        let count = call_count.clone();
        let r1 = cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        let count = call_count.clone();
        let r2 = cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        // Second call should use cache
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert!(Arc::ptr_eq(&r1, &r2));
    }

    #[tokio::test]
    async fn cache_expires_after_ttl() {
        let cache = QueryCache::with_ttl(Duration::from_millis(50));
        let call_count = Arc::new(AtomicU32::new(0));

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn invalidate_clears_cache() {
        let cache = QueryCache::new();
        let call_count = Arc::new(AtomicU32::new(0));

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        cache.invalidate_all().await;

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
