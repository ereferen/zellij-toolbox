//! Version detection cache for tool version results
//!
//! Provides in-memory caching with optional file persistence to avoid
//! redundant version command executions.

use crate::info::ToolInfo;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default TTL for cache entries (5 minutes)
const DEFAULT_TTL_SECONDS: u64 = 300;

/// A single cached version detection result
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached tool info
    pub tool_info: ToolInfo,
    /// Unix timestamp when this entry was created
    pub detected_at: u64,
    /// Working directory at detection time
    pub working_dir: Option<String>,
    /// Time-to-live in seconds for this entry
    pub ttl_seconds: u64,
}

impl CacheEntry {
    /// Check if this cache entry has expired
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        now.saturating_sub(self.detected_at) > self.ttl_seconds
    }

    /// Check if this entry matches the given working directory
    pub fn matches_working_dir(&self, working_dir: &Option<String>) -> bool {
        self.working_dir == *working_dir
    }
}

/// In-memory cache for tool version detection results
#[derive(Debug)]
pub struct VersionCache {
    entries: HashMap<String, CacheEntry>,
    default_ttl: u64,
    /// Statistics: number of cache hits
    hits: u64,
    /// Statistics: number of cache misses
    misses: u64,
}

impl Default for VersionCache {
    fn default() -> Self {
        Self::new(DEFAULT_TTL_SECONDS)
    }
}

impl VersionCache {
    /// Create a new cache with the given default TTL (in seconds)
    pub fn new(default_ttl: u64) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl,
            hits: 0,
            misses: 0,
        }
    }

    /// Look up a cached result for the given tool name and working directory.
    /// Returns `None` if not found, expired, or working directory doesn't match.
    pub fn get(&mut self, tool_name: &str, working_dir: &Option<String>) -> Option<&ToolInfo> {
        // Check if entry exists and is valid
        let valid = self
            .entries
            .get(tool_name)
            .map(|entry| !entry.is_expired() && entry.matches_working_dir(working_dir))
            .unwrap_or(false);

        if valid {
            self.hits += 1;
            self.entries.get(tool_name).map(|e| &e.tool_info)
        } else {
            self.misses += 1;
            // Remove expired entry if present
            if self.entries.contains_key(tool_name) {
                self.entries.remove(tool_name);
            }
            None
        }
    }

    /// Store a detection result in the cache
    pub fn put(&mut self, tool_name: String, tool_info: ToolInfo, working_dir: Option<String>) {
        self.put_with_ttl(tool_name, tool_info, working_dir, self.default_ttl);
    }

    /// Store a detection result with a specific TTL
    pub fn put_with_ttl(
        &mut self,
        tool_name: String,
        tool_info: ToolInfo,
        working_dir: Option<String>,
        ttl_seconds: u64,
    ) {
        let entry = CacheEntry {
            tool_info,
            detected_at: current_timestamp(),
            working_dir,
            ttl_seconds,
        };
        self.entries.insert(tool_name, entry);
    }

    /// Invalidate all entries (clear the entire cache)
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Invalidate a specific tool's cache entry
    pub fn invalidate(&mut self, tool_name: &str) {
        self.entries.remove(tool_name);
    }

    /// Remove all expired entries
    pub fn evict_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    /// Get the number of entries currently in the cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache hit count
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Get cache miss count
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// Get cache hit rate as a percentage (0.0 - 100.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Reset statistics counters
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    /// Get the default TTL
    pub fn default_ttl(&self) -> u64 {
        self.default_ttl
    }
}

/// Get current unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool_info(name: &str, version: &str) -> ToolInfo {
        ToolInfo::available(name.to_string(), version.to_string())
    }

    // --- VersionCache basic operations ---

    #[test]
    fn test_cache_new() {
        let cache = VersionCache::new(60);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.default_ttl(), 60);
    }

    #[test]
    fn test_cache_default() {
        let cache = VersionCache::default();
        assert_eq!(cache.default_ttl(), DEFAULT_TTL_SECONDS);
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = VersionCache::new(300);
        let info = make_tool_info("Python", "3.12.0");
        let working_dir = Some("/home/user/project".to_string());

        cache.put("Python".to_string(), info.clone(), working_dir.clone());
        assert_eq!(cache.len(), 1);

        let result = cache.get("Python", &working_dir);
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, Some("3.12.0".to_string()));
    }

    #[test]
    fn test_cache_miss_nonexistent() {
        let mut cache = VersionCache::new(300);
        let result = cache.get("Node", &None);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_working_dir_mismatch() {
        let mut cache = VersionCache::new(300);
        let info = make_tool_info("Python", "3.12.0");

        cache.put(
            "Python".to_string(),
            info,
            Some("/home/user/project-a".to_string()),
        );

        // Different working dir should miss
        let result = cache.get("Python", &Some("/home/user/project-b".to_string()));
        assert!(result.is_none());

        // Mismatched entry should be removed
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_working_dir_none_matches_none() {
        let mut cache = VersionCache::new(300);
        let info = make_tool_info("Python", "3.12.0");

        cache.put("Python".to_string(), info, None);

        let result = cache.get("Python", &None);
        assert!(result.is_some());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = VersionCache::new(300);
        cache.put(
            "Python".to_string(),
            make_tool_info("Python", "3.12.0"),
            None,
        );
        cache.put("Node".to_string(), make_tool_info("Node", "20.10.0"), None);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_invalidate_single() {
        let mut cache = VersionCache::new(300);
        cache.put(
            "Python".to_string(),
            make_tool_info("Python", "3.12.0"),
            None,
        );
        cache.put("Node".to_string(), make_tool_info("Node", "20.10.0"), None);

        cache.invalidate("Python");
        assert_eq!(cache.len(), 1);
        assert!(cache.get("Python", &None).is_none());
        assert!(cache.get("Node", &None).is_some());
    }

    // --- Expiration tests ---

    #[test]
    fn test_cache_entry_not_expired() {
        let entry = CacheEntry {
            tool_info: make_tool_info("Python", "3.12.0"),
            detected_at: current_timestamp(),
            working_dir: None,
            ttl_seconds: 300,
        };
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expired() {
        let entry = CacheEntry {
            tool_info: make_tool_info("Python", "3.12.0"),
            detected_at: current_timestamp().saturating_sub(600), // 10 minutes ago
            working_dir: None,
            ttl_seconds: 300, // 5 minute TTL
        };
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_expired_entry_returns_none() {
        let mut cache = VersionCache::new(0); // TTL of 0 seconds = immediately expire
        cache.put(
            "Python".to_string(),
            make_tool_info("Python", "3.12.0"),
            None,
        );

        // With TTL=0, entry should already be expired
        // We need to wait at least 1 second, but instead we can manually set detected_at
        // Just test that the entry is removed on next get
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let result = cache.get("Python", &None);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_evict_expired() {
        let mut cache = VersionCache::new(300);

        // Add an entry that's already expired (by manipulating directly)
        cache.entries.insert(
            "OldTool".to_string(),
            CacheEntry {
                tool_info: make_tool_info("OldTool", "1.0.0"),
                detected_at: current_timestamp().saturating_sub(600),
                working_dir: None,
                ttl_seconds: 300,
            },
        );

        // Add a fresh entry
        cache.put("Fresh".to_string(), make_tool_info("Fresh", "2.0.0"), None);

        assert_eq!(cache.len(), 2);
        cache.evict_expired();
        assert_eq!(cache.len(), 1);
        assert!(cache.get("Fresh", &None).is_some());
    }

    // --- Statistics tests ---

    #[test]
    fn test_cache_stats_initial() {
        let cache = VersionCache::new(300);
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_tracking() {
        let mut cache = VersionCache::new(300);
        cache.put(
            "Python".to_string(),
            make_tool_info("Python", "3.12.0"),
            None,
        );

        // Hit
        cache.get("Python", &None);
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 0);

        // Miss
        cache.get("Node", &None);
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);

        // Hit rate = 50%
        assert!((cache.hit_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_reset() {
        let mut cache = VersionCache::new(300);
        cache.put(
            "Python".to_string(),
            make_tool_info("Python", "3.12.0"),
            None,
        );
        cache.get("Python", &None);
        cache.get("Node", &None);

        cache.reset_stats();
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
    }

    // --- CacheEntry tests ---

    #[test]
    fn test_cache_entry_matches_working_dir() {
        let entry = CacheEntry {
            tool_info: make_tool_info("Python", "3.12.0"),
            detected_at: current_timestamp(),
            working_dir: Some("/home/user/project".to_string()),
            ttl_seconds: 300,
        };

        assert!(entry.matches_working_dir(&Some("/home/user/project".to_string())));
        assert!(!entry.matches_working_dir(&Some("/other/dir".to_string())));
        assert!(!entry.matches_working_dir(&None));
    }

    #[test]
    fn test_cache_entry_none_working_dir() {
        let entry = CacheEntry {
            tool_info: make_tool_info("Python", "3.12.0"),
            detected_at: current_timestamp(),
            working_dir: None,
            ttl_seconds: 300,
        };

        assert!(entry.matches_working_dir(&None));
        assert!(!entry.matches_working_dir(&Some("/some/dir".to_string())));
    }

    // --- put_with_ttl tests ---

    #[test]
    fn test_cache_put_with_custom_ttl() {
        let mut cache = VersionCache::new(300);
        cache.put_with_ttl(
            "Python".to_string(),
            make_tool_info("Python", "3.12.0"),
            None,
            60, // 1 minute TTL
        );

        let entry = cache.entries.get("Python").unwrap();
        assert_eq!(entry.ttl_seconds, 60);
    }

    #[test]
    fn test_cache_overwrite_entry() {
        let mut cache = VersionCache::new(300);
        cache.put(
            "Python".to_string(),
            make_tool_info("Python", "3.11.0"),
            None,
        );
        cache.put(
            "Python".to_string(),
            make_tool_info("Python", "3.12.0"),
            None,
        );

        assert_eq!(cache.len(), 1);
        let result = cache.get("Python", &None);
        assert_eq!(result.unwrap().version, Some("3.12.0".to_string()));
    }
}
