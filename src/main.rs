use std::fs::File;
use std::io::Write;

use clap::Parser;
use std::path::PathBuf;

use pack_it_up::offline::first_fit_decreasing::first_fit_decreasing;
use pack_it_up::Pack;
use walkdir::{Result, WalkDir};

struct FileItem {
    file_path: PathBuf,
    size: usize,
}

impl Pack for FileItem {
    fn size(&self) -> usize {
        self.size
    }
}

/// Create a bash script to distribute several files into multiple directories of maximum size, similar to disc spanning.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the directory containing the files to distribute into equal-sized directories.
    #[arg(short, long)]
    src: PathBuf,

    /// Root path to the directories containing the output files. This will create subdirectories named "disk000", "disk001", etc.
    #[arg(short, long)]
    dest: PathBuf,

    /// Desired size of each disc in bytes.
    #[arg(long = "size", default_value = "100000000000")]
    disc_size: usize,
}

fn get_all_files(path: &PathBuf) -> Result<Vec<FileItem>> {
    let mut filenames_and_sizes: Vec<FileItem> = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let metadata = entry.metadata().unwrap();
        let filename = entry.path();
        let size = metadata.len();

        filenames_and_sizes.push(FileItem {
            file_path: filename.to_path_buf(),
            size: size as usize,
        });
    }

    Ok(filenames_and_sizes)
}

fn write_results(bins: Vec<pack_it_up::Bin<FileItem>>, disc_size: usize) -> std::io::Result<()> {
    let mut file = File::create("move_files.sh")?;
    file.write_all("#!/bin/env bash".as_bytes())?;

    for (k, bin) in bins.iter().enumerate() {
        let output_dir = format!("disk{k:03}/");
        file.write_all(format!("# These files will be moved to {output_dir}\r\n").as_bytes())?;
        let mut total_bin_size = 0;
        for bin_content in bin.contents() {
            total_bin_size += bin_content.size;
            file.write_all(
                format!(
                    r#"mv "{0}" "{output_dir}"
"#,
                    bin_content.file_path.display()
                )
                .as_bytes(),
            )?;
        }
        let num_files = bin.contents().len();
        let unused_space = disc_size - total_bin_size;
        println!("Disc {k:3}: {num_files} files, total size {total_bin_size} bytes, unused space {unused_space} bytes.");
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let all_files = get_all_files(&args.src)?;

    // Perform bin packing into fixed-size bins. This will create new bins automatically.
    let bins = first_fit_decreasing(args.disc_size, all_files);

    write_results(bins, args.disc_size)
}
