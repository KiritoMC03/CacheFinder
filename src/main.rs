extern crate fs_extra;
extern crate walkdir;

use std::cmp::Ordering;
use std::fs::{self, ReadDir};
use std::path::Path;
use walkdir::WalkDir;

const CACHE_PATTERN: &str = "cache";
const MIN_SIZE_BYTES: u64 = 100 * 1024 * 1024;

const ROOT_DIR_PATHS: &'static [&'static str] = &["C:\\Users\\kiril", "C:\\Program Files (x86)"];

struct CacheSearchingResult {
    folders: Vec<CacheFolderInfo>,
    stack_capacity: usize,
}

struct CacheFolderInfo {
    path: String,
    size: u64,
}

impl CacheFolderInfo {
    fn new(path: &String) -> CacheFolderInfo {
        CacheFolderInfo { path: path.clone(), size: dir_size(&path) }
    }
}

fn main() {
    println!("Searching...");
    let mut dirs_dor_check: Vec<ReadDir> = Vec::with_capacity(ROOT_DIR_PATHS.len());
    ROOT_DIR_PATHS
        .into_iter()
        .filter_map(|path| fs::read_dir(path).ok())
        .for_each(|item| dirs_dor_check.push(item));

    find_cache(dirs_dor_check).map(print_caches);

    println!("Finished!");
    loop{}
}

fn find_cache(mut root_dirs: Vec<ReadDir>) -> Option<CacheSearchingResult> {
    let mut stack: Vec<String> = Vec::with_capacity(128);
    let mut cache_paths: Vec<CacheFolderInfo> = Vec::with_capacity(100);
    root_dirs.drain(..)
        .for_each(|dir| push_children_to_stack(dir, &mut stack));
    while stack.len() > 0 {
        let current_path = stack.pop().unwrap();
        if current_path.to_lowercase().contains(CACHE_PATTERN) {
            cache_paths.push(CacheFolderInfo::new(&current_path));
        } else {
            match fs::read_dir(current_path) {
                Ok(dir) => push_children_to_stack(dir, &mut stack),
                Err(_) => {}
            }
        }
    }

    Some(CacheSearchingResult {
        folders: cache_paths,
        stack_capacity: stack.capacity(),
    })
}

fn push_children_to_stack(root: ReadDir, stack: &mut Vec<String>) {
    for dir in root.into_iter() {
        if let Ok(entry) = dir {
            if let Some(path) = entry.path().to_str() {
                if is_large_dir(&path) {
                    stack.push(path.to_string())
                };
            }
        }
    }
}

fn is_large_dir<P: AsRef<Path>>(path: &P) -> bool {
    dir_size(path) > MIN_SIZE_BYTES
}

fn print_caches(mut result: CacheSearchingResult) {
    result.folders.sort_by(|a, b| {
        if a.size > b.size { return Ordering::Less }
        Ordering::Greater
    });
    for cache_info in &result.folders {
        println!("{}mb -> {}", bytes_to_mb(cache_info.size), cache_info.path);
    }

    println!("\nStack capacity: {}", result.stack_capacity);
    println!("Size sum: {}mb", bytes_to_mb(get_size_sum(&result.folders)));
}

fn dir_size<P: AsRef<Path>>(path: &P) -> u64 {
    WalkDir::new(path)
        .min_depth(1)
        .max_depth(100)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .fold(0, |acc, m| acc + m.len())
}

fn bytes_to_mb(bytes: u64) -> u64 {
    bytes / (1024 * 1025)
}

fn get_size_sum(folders: &Vec<CacheFolderInfo>) -> u64{
    let mut sum = 0u64;
    for folder in folders {
        sum += folder.size;
    }
    sum
}