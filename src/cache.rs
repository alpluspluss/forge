use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use log::{debug, trace};
use crate::error::{ForgeError, ForgeResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    hash: String,
    includes: HashMap<PathBuf, FileInfo>,
    compiler_flags: Vec<String>,
    target: String,
    profile: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    hash: String,
    mtime: u64,
    size: u64,
}

pub struct BuildCache {
    cache_dir: PathBuf,
    entries: HashMap<PathBuf, CacheEntry>,
    quick_check: bool,
}

impl BuildCache {
    pub fn new(workspace_root: &Path) -> Self {
        let cache_dir = workspace_root.join(".forge_cache");
        fs::create_dir_all(&cache_dir).ok();

        BuildCache {
            cache_dir,
            entries: HashMap::new(),
            quick_check: true,
        }
    }

    pub fn needs_rebuild(
        &self,
        source: &Path,
        object: &Path,
        includes: &[PathBuf],
        compiler_flags: &[String],
        target: &str,
        profile: &str,
    ) -> bool {
        debug!("Checking if {:?} needs rebuild...", source);

        if !object.exists() {
            debug!("Object file doesn't exist");
            return true;
        }

        if let Some(entry) = self.entries.get(source) {
            if entry.target != target ||
                entry.profile != profile ||
                entry.compiler_flags != compiler_flags {
                debug!("Build configuration changed");
                return true;
            }

            if self.file_changed(source, &entry.hash) {
                debug!("Source file changed");
                return true;
            }

            for include in includes {
                if let Some(info) = entry.includes.get(include) {
                    if self.file_changed_with_info(include, info) {
                        debug!("Include file {:?} changed", include);
                        return true;
                    }
                } else {
                    debug!("New include file {:?}", include);
                    return true;
                }
            }

            if entry.includes.len() != includes.len() {
                debug!("Number of includes changed");
                return true;
            }

            false
        } else {
            debug!("No cache entry found");
            true
        }
    }

    pub fn update(
        &mut self,
        source: &Path,
        includes: &[PathBuf],
        compiler_flags: &[String],
        target: &str,
        profile: &str,
    ) -> ForgeResult<()> {
        let mut include_infos = HashMap::new();

        for include in includes {
            include_infos.insert(
                include.to_path_buf(),
                self.get_file_info(include)?,
            );
        }

        self.entries.insert(
            source.to_path_buf(),
            CacheEntry {
                hash: self.get_file_info(source)?.hash,
                includes: include_infos,
                compiler_flags: compiler_flags.to_vec(),
                target: target.to_string(),
                profile: profile.to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        );

        Ok(())
    }

    fn get_file_info(&self, path: &Path) -> ForgeResult<FileInfo> {
        let metadata = fs::metadata(path)
            .map_err(|e| ForgeError::Cache(format!("Failed to get metadata for {}: {}", path.display(), e)))?;

        Ok(FileInfo {
            hash: if self.quick_check {
                "quick_check".to_string()
            } else {
                self.hash_file(path)?
            },
            mtime: metadata.modified()
                .unwrap_or(UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            size: metadata.len(),
        })
    }

    fn file_changed(&self, path: &Path, old_hash: &str) -> bool {
        if let Ok(info) = self.get_file_info(path) {
            if self.quick_check {
                trace!("Quick check for {:?}", path);
                false
            } else {
                info.hash != old_hash
            }
        } else {
            true
        }
    }

    fn file_changed_with_info(&self, path: &Path, old_info: &FileInfo) -> bool {
        if let Ok(new_info) = self.get_file_info(path) {
            if self.quick_check {
                // First do a quick mtime/size check
                if new_info.mtime != old_info.mtime || new_info.size != old_info.size {
                    debug!("Quick check detected change in {:?}", path);
                    true
                } else {
                    false
                }
            } else {
                new_info.hash != old_info.hash
            }
        } else {
            true
        }
    }

    fn hash_file(&self, path: &Path) -> ForgeResult<String> {
        let mut hasher = Sha256::new();
        let contents = fs::read(path)
            .map_err(|e| ForgeError::Cache(format!("Failed to read {}: {}", path.display(), e)))?;

        hasher.update(&contents);
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn save(&self) -> ForgeResult<()> {
        for (path, entry) in &self.entries {
            let cache_path = self.cache_dir.join(format!(
                "{}.cache",
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ));

            let content = serde_json::to_string(entry)
                .map_err(|e| ForgeError::Cache(format!("Failed to serialize cache: {}", e)))?;

            fs::write(&cache_path, content)
                .map_err(|e| ForgeError::Cache(format!("Failed to write cache: {}", e)))?;
        }
        Ok(())
    }

    pub fn load(&mut self) -> ForgeResult<()> {
        for entry in fs::read_dir(&self.cache_dir)
            .map_err(|e| ForgeError::Cache(format!("Failed to read cache directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| ForgeError::Cache(format!("Failed to read cache entry: {}", e)))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "cache") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| ForgeError::Cache(format!("Failed to read cache file: {}", e)))?;

                let cache_entry: CacheEntry = serde_json::from_str(&content)
                    .map_err(|e| ForgeError::Cache(format!("Failed to parse cache: {}", e)))?;

                let source_name = path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                self.entries.insert(PathBuf::from(source_name), cache_entry);
            }
        }
        Ok(())
    }

    pub fn set_quick_check(&mut self, enable: bool) {
        self.quick_check = enable;
    }

    pub fn clean(&self) -> ForgeResult<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| ForgeError::Cache(format!("Failed to remove cache directory: {}", e)))?;
        }
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| ForgeError::Cache(format!("Failed to create cache directory: {}", e)))?;
        Ok(())
    }
}