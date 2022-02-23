use colored::*;
use log::{debug, error, info};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Deserialize)]
struct Config {
    pub interval_seconds: u64,
    pub directory: String,
    pub max_size_mb: u64,
}

fn main() {
    env_logger::init();

    let config = envy::from_env::<Config>().unwrap();
    let directory = Path::new(&config.directory);

    println!(
        "{}",
        format!(
            "Starting {APP_NAME}-v{VERSION} and running every {} seconds on {} with a limit of {} MB",
            config.interval_seconds,
            directory.display(),
            config.max_size_mb
        )
        .magenta()
    );

    loop {
        info!(
            "{}",
            format!(
                "Running cleanup loop, every {} seconds",
                config.interval_seconds
            )
            .green()
        );

        match read_dir(directory) {
            Ok(files) => process(files, config.max_size_mb, directory),
            Err(e) => {
                info!(
                    "{}",
                    format!("Error while reading {directory:?}: {e:?}").red()
                );
            }
        }

        std::thread::sleep(Duration::from_secs(config.interval_seconds));
    }
}

struct FileInfo {
    path: PathBuf,
    size: u64,
    modified: u64,
}

struct ReadDirResult {
    files: Vec<FileInfo>,
    total_size: u64,
}

fn read_dir(path: &Path) -> std::io::Result<ReadDirResult> {
    let mut files = Vec::new();
    let mut total_size = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = std::fs::metadata(&path);

        if let Ok(metadata) = metadata {
            if metadata.is_dir() {
                let mut items = read_dir(&path)?;
                files.append(&mut items.files);
                total_size += items.total_size;
            } else {
                total_size += metadata.len();

                let modified = match metadata.modified() {
                    Ok(val) => val,
                    Err(_) => metadata.created().expect("created timestamp not available"),
                };
                files.push(FileInfo {
                    path,
                    size: metadata.len(),
                    modified: modified.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                });
            }
        } else {
            error!(
                "{}",
                format!(
                    "Error getting file metadata: {}, {metadata:?}",
                    path.display()
                )
                .red()
            );
        }
    }

    Ok(ReadDirResult { files, total_size })
}

fn process(data: ReadDirResult, max_size_mb: u64, directory: &Path) {
    let mut parent_dirs_files_count = HashMap::new();

    for file in &data.files {
        file.path.ancestors().skip(1).for_each(|component| {
            if !component.starts_with(directory) || component == directory {
                return;
            }

            let mut path = PathBuf::new();
            path.push(component);

            *parent_dirs_files_count.entry(path).or_insert(0) += 1;
        });
    }

    let max_size_bytes = max_size_mb * 1024 * 1024;
    let total_files = data.files.len();
    let mut total_size = data.total_size;
    let total_size_mb = total_size as f64 / 1024.0 / 1024.0;

    if total_size < max_size_bytes {
        info!(
            "{}",
            format!(
                "Total size: {total_size_mb:.2} MB in {total_files} files, limit set to {} MB",
                max_size_mb
            )
            .green()
        );
        return;
    }

    info!(
            "{}",
            format!(
                "Total size: {total_size_mb:.2} MB in {total_files} files is greater than max size of {max_size_mb} MB... doing cleanup of older files",
            )
            .red()
        );

    let mut sorted_files = data.files;
    sorted_files.sort_by(|a, b| b.modified.cmp(&a.modified));

    while total_size > max_size_bytes {
        let file = sorted_files.pop();

        if file.is_none() {
            break;
        }

        let file = file.unwrap();

        if let Err(e) = std::fs::remove_file(&file.path) {
            error!(
                "{}",
                format!("Error removing file: {}, {e:?}", file.path.display()).red()
            );
        } else {
            debug!("{}", format!("Removed file: {}", file.path.display()).red());
        }

        total_size = total_size - file.size;
    }
}
