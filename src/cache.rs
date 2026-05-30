use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::scanner::DirNode;

#[derive(Serialize, Deserialize)]
struct CacheFile {
    scanned_at: SystemTime,
    root: DirNode,
}

pub fn load(scan_path: &Path, max_age_hours: u64) -> Option<(DirNode, SystemTime)> {
    let path = cache_path(scan_path).ok()?;
    let data = std::fs::read_to_string(path).ok()?;
    let cf: CacheFile = serde_json::from_str(&data).ok()?;

    let age = SystemTime::now().duration_since(cf.scanned_at).ok()?;
    if age > Duration::from_secs(max_age_hours * 3600) {
        return None;
    }

    Some((cf.root, cf.scanned_at))
}

pub fn save(scan_path: &Path, root: &DirNode) -> Result<()> {
    let path = cache_path(scan_path)?;
    let cf = CacheFile {
        scanned_at: SystemTime::now(),
        root: root.clone(),
    };
    std::fs::write(path, serde_json::to_string(&cf)?)?;
    Ok(())
}

pub fn invalidate(scan_path: &Path) {
    if let Ok(p) = cache_path(scan_path) {
        let _ = std::fs::remove_file(p);
    }
}

fn cache_path(scan_path: &Path) -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot locate data directory"))?
        .join("diskview");
    std::fs::create_dir_all(&dir)?;

    let mut hasher = DefaultHasher::new();
    scan_path.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(dir.join(format!("{:016x}.json", hash)))
}
