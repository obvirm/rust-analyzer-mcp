use std::path::{Path, PathBuf};

pub fn find_cargo_project(start_dir: &Path) -> Option<PathBuf> {
    let mut current = Some(start_dir.to_path_buf());

    while let Some(dir) = current {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }

    None
}

pub fn find_all_projects(root: &Path) -> Vec<PathBuf> {
    let mut projects = Vec::new();

    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let cargo_toml = path.join("Cargo.toml");
                if cargo_toml.exists() {
                    projects.push(path);
                } else {
                    projects.extend(find_all_projects(&path));
                }
            }
        }
    }

    projects
}

pub fn is_rust_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "rs")
        .unwrap_or(false)
}
