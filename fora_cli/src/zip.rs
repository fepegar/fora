// Cargo.toml
// [package]
// name = "zip-in-memory"
// version = "0.1.0"
// edition = "2021"
//
// [dependencies]
// zip = "0.6"
// ignore = "0.4"
// rayon = "1.7"
// reqwest = { version = "0.11", features = ["rustls-tls"] }
// tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

// ----------------- Full example: parallel read + sequential ZIP + async PUT -----------------

use ignore::WalkBuilder;
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use tokio::task;
use zip::write::FileOptions;
use zip::CompressionMethod;

/// Walk `dir` respecting .gitignore and collect regular file paths.
fn collect_file_paths(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let walker = WalkBuilder::new(dir).git_ignore(true).build();

    let mut paths = Vec::new();
    for entry in walker {
        let e = entry?;
        if e.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            paths.push(e.into_path());
        }
    }
    Ok(paths)
}

/// Read the collected files in parallel into memory.
/// Returns a Vec of (zip_entry_name, file_bytes).
fn read_files_parallel(dir: PathBuf, paths: Vec<PathBuf>) -> Vec<(String, Vec<u8>)> {
    paths
        .into_par_iter()
        .map(|path| {
            // Read file into memory
            let mut f = File::open(&path).expect("failed to open file");
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).expect("failed to read file");

            // Compute name inside zip relative to `dir` with forward slashes
            let rel = path
                .strip_prefix(&dir)
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            let name = rel.replace(std::path::MAIN_SEPARATOR, "/");

            (name, buf)
        })
        .collect()
}

/// Given an iterator of (name, bytes), create an in-memory zip archive (Vec<u8>).
/// This function is synchronous and uses zip::ZipWriter sequentially.
fn zip_from_files(
    files: Vec<(String, Vec<u8>)>,
    compress: bool,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut buffer);

    let options = if compress {
        FileOptions::default().compression_method(CompressionMethod::Deflated)
    } else {
        FileOptions::default().compression_method(CompressionMethod::Stored)
    };

    for (name, data) in files {
        // Start file entry in the zip and write contents
        zip.start_file(name, options)?;
        zip.write_all(&data)?;
    }

    zip.finish()?;
    Ok(buffer.into_inner())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Settings you can change
    let dir = Path::new("./my_folder").to_path_buf();
    let upload_url = "https://example.com/upload"; // change to your target
    let compress = true; // set to false to use Stored (no compression) - faster to create

    // 1) Collect file paths (this is cheap and synchronous)
    let paths = collect_file_paths(&dir)?;

    // 2) Spawn blocking: read files in parallel (Rayon) and build the zip synchronously.
    //    We do this inside `spawn_blocking` so the async runtime is not blocked.
    let dir_clone = dir.clone();
    let zip_bytes = task::spawn_blocking(move || {
        // Read files in parallel into memory
        let files = read_files_parallel(dir_clone, paths);
        // Then create zip sequentially (ZipWriter does compression while writing)
        zip_from_files(files, compress)
    })
    .await??; // unwrap JoinError and inner Result

    println!("Created zip in memory: {} bytes", zip_bytes.len());

    // 3) Upload the zip bytes asynchronously with reqwest
    let client = reqwest::Client::new();
    let resp = client
        .put(upload_url)
        .header("Content-Type", "application/zip")
        .body(zip_bytes)
        .send()
        .await?;

    println!("Upload finished: {}", resp.status());

    Ok(())
}

// ---------------- Notes / tuning ----------------
// * Memory: this approach reads all selected files into memory. Make sure you have
//   enough RAM for the total size of files + zip overhead. If memory is a concern,
//   consider streaming or chunking (process N files at a time).
// * Compression: Setting `compress = false` uses Stored mode which is much faster to
//   produce and useful when files are already compressed or CPU is a bottleneck.
// * Parallelism: Rayon uses a global thread pool. If you want to limit threads, set
//   RAYON_NUM_THREADS or configure a custom pool.
// * Error handling: the example uses `expect` inside the parallel iterator for clarity.
//   Swap for proper error propagation if you need robust failures across many files.
// * Streaming uploads: this example holds the full ZIP in memory. If you want to stream
//   directly while producing the archive to save peak memory, that requires an async
//   streaming integration (more complex). I can provide that if you need it.

// --------------------------------------------------------------------------------
