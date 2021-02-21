
use std::path::PathBuf;
use walkdir::{WalkDir, DirEntry};
use globset::Glob;
use indicatif::ProgressBar;
use clap::{Arg, App};

use anyhow::{Context, Result, bail};

mod process;
mod context;

fn main() -> Result<()> {
    let matches = App::new("rydde")
        .version("0.1.0")
        .author("Mel Teichmann <mel@teichmann.dev>")
        .about("Sorts JPEGs and PNGs into folders using the exif creation date")
        .arg(Arg::with_name("src").required(true).value_name("SRC"))
        .arg(Arg::with_name("dst").required(true).value_name("DST"))
        .arg(
            Arg::with_name("filter")
                .long("filter")
                .short("f")
                .default_value("*.{jpg,jpeg,png,tiff}")
        )
        .arg(
            Arg::with_name("folder_tpl")
                .long("path-template")
                .short("p")
                .default_value("%Y/%B/")
        )
        .arg(
            Arg::with_name("file_tpl")
                .long("name-template")
                .short("n")
                .default_value("%Y%m%d%H%M%S")
        )
        .get_matches();

    let filter_arg = matches.value_of("filter").unwrap();
    let src_dir_arg = PathBuf::from(matches.value_of("src").unwrap());
    if !src_dir_arg.exists() {
        bail!("SRC folder {} does not exist", src_dir_arg.to_string_lossy());
    }
    let dst_dir_arg = PathBuf::from(matches.value_of("dst").unwrap());

    let file_tpl_arg = matches.value_of("file_tpl").unwrap();
    let folder_tpl_arg = matches.value_of("folder_tpl").unwrap();
    
    let glob = Glob::new(filter_arg).context("Failed to parse glob pattern")?.compile_matcher();
    let files : Vec<DirEntry> = WalkDir::new(&src_dir_arg).into_iter().filter_map(|e| e.ok()).filter(|e| glob.is_match(e.path())).collect();


    println!("Src: {}", src_dir_arg.to_string_lossy());
    println!("Dst: {}", dst_dir_arg.to_string_lossy());
    println!("Found {} images", files.len());

    let bar = ProgressBar::new(files.len() as u64);

    let result = process::run(files, &dst_dir_arg, file_tpl_arg, folder_tpl_arg, bar)?;
    println!("{}", result);

    Ok(())
}
