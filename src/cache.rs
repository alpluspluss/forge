// src/cache.rs
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
    time::SystemTime,
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    hash: String,
    includes: HashMap<String, String>,
    timestamp: u64,
}

pub struct BuildCache {
    cache_dir: PathBuf,
    entries: HashMap<PathBuf, CacheEntry>,
}

impl BuildCache {
    pub fn new(workspace_root: &Path) -> Self {
        let cache_dir = workspace_root.join(".forge_cache");
        fs::create_dir_all(&cache_dir).ok();

        BuildCache {
            cache_dir,
            entries: HashMap::new(),
        }
    }

    pub fn needs_rebuild(&self, source: &Path, object: &Path, includes: &[PathBuf]) -> bool {
        // If object doesn't exist, needs rebuild
        if !object.exists() {
            return true;
        }

        // Check source file hash
        let current_hash = self.hash_file(source);
        if let Some(entry) = self.entries.get(source) {
            if entry.hash != current_hash {
                return true;
            }

            // Check includes
            for include in includes {
                let include_str = include.to_string_lossy().to_string();
                if let Some(old_hash) = entry.includes.get(&include_str) {
                    if self.hash_file(include) != *old_hash {
                        return true;
                    }
                } else {
                    return true;
                }
            }

            false
        } else {
            true
        }
    }

    pub fn update(&mut self, source: &Path, includes: &[PathBuf]) {
        let mut include_hashes = HashMap::new();
        for include in includes {
            include_hashes.insert(
                include.to_string_lossy().to_string(),
                self.hash_file(include),
            );
        }

        self.entries.insert(
            source.to_path_buf(),
            CacheEntry {
                hash: self.hash_file(source),
                includes: include_hashes,
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        );
    }

    fn hash_file(&self, path: &Path) -> String {
        let mut hasher = Sha256::new();
        if let Ok(contents) = fs::read(path) {
            hasher.update(&contents);
            format!("{:x}", hasher.finalize())
        } else {
            String::new()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        for (path, entry) in &self.entries {
            let cache_path = self.cache_dir.join(
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string() + ".cache"
            );

            let content = serde_json::to_string(entry)
                .map_err(|e| format!("Failed to serialize cache: {}", e))?;

            fs::write(&cache_path, content)
                .map_err(|e| format!("Failed to write cache: {}", e))?;
        }
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        for entry in fs::read_dir(&self.cache_dir)
            .map_err(|e| format!("Failed to read cache directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read cache entry: {}", e))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "cache") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read cache file: {}", e))?;

                let cache_entry: CacheEntry = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse cache: {}", e))?;

                let source_name = path.file_stem().unwrap().to_string_lossy().to_string();
                self.entries.insert(PathBuf::from(source_name), cache_entry);
            }
        }
        Ok(())
    }
}