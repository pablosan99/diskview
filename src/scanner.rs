use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub file_count: u64,
    pub children: Vec<DirNode>,
}

pub fn scan(path: &Path, progress: Arc<AtomicU64>) -> Result<DirNode> {
    let node = scan_dir(path, &progress, true);
    Ok(node)
}

fn scan_dir(path: &Path, progress: &Arc<AtomicU64>, is_root: bool) -> DirNode {
    progress.fetch_add(1, Ordering::Relaxed);

    let name = if is_root {
        path.to_string_lossy().into_owned()
    } else {
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned())
    };

    let entries: Vec<_> = match std::fs::read_dir(path) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => {
            return DirNode {
                name,
                path: path.to_path_buf(),
                size: 0,
                file_count: 0,
                children: vec![],
            };
        }
    };

    let mut direct_size: u64 = 0;
    let mut direct_file_count: u64 = 0;
    let mut subdirs: Vec<PathBuf> = vec![];

    for entry in &entries {
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => subdirs.push(entry.path()),
            Ok(_) => {
                direct_file_count += 1;
                if let Ok(meta) = entry.metadata() {
                    direct_size += meta.len();
                }
            }
            _ => {}
        }
    }

    let mut children: Vec<DirNode> = subdirs
        .par_iter()
        .map(|p| scan_dir(p, progress, false))
        .collect();

    children.sort_by(|a, b| b.size.cmp(&a.size));

    let child_size: u64 = children.iter().map(|c| c.size).sum();
    let child_file_count: u64 = children.iter().map(|c| c.file_count).sum();

    DirNode {
        name,
        path: path.to_path_buf(),
        size: direct_size + child_size,
        file_count: direct_file_count + child_file_count,
        children,
    }
}
