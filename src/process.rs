use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom};
use std::str;

use walkdir::DirEntry;
use rayon::prelude::*;
use chrono::NaiveDateTime;
use blake2::VarBlake2b;
use exif::{Exif, Tag, In, Value};
use blake2::digest::{Update, VariableOutput};
use indicatif::ProgressBar;
use anyhow::{Result};
use std::sync::Arc;
use crate::context::{Context, ProcessResult};


fn get_date_time(exif: Exif) -> Result<Option<NaiveDateTime>> {
    match exif.get_field(Tag::DateTime, In::PRIMARY) {
        Some(field) => match field.value {
            Value::Ascii(ref array) => match array.get(0) {
                Some(date_byte_string) => Ok(Some(NaiveDateTime::parse_from_str(str::from_utf8(date_byte_string)?, "%Y:%m:%d %H:%M:%S")?)),
                None => Ok(None)
            },
            _ => Ok(None)
        },
        None => Ok(None)
    }
}

pub fn run(files: Vec<DirEntry>, dst_dir: &Path, file_tpl: &str, folder_tpl: &str, bar: ProgressBar) -> Result<Arc<ProcessResult>> {
    let unknown_path = dst_dir.join(Path::new("unknown"));
    fs::create_dir_all(unknown_path.clone())?;

    let result = Arc::new(ProcessResult::new());

    files.par_iter().try_for_each_init(|| Context::new(&bar, &result), |ctx, x| -> Result<()> {
        ctx.buffer.seek(SeekFrom::Start(0))?;

        let src_path = x.path();
        let mut file = File::open(src_path)?;
        let mut hasher = VarBlake2b::new(10)?;
        
        io::copy(&mut file, &mut ctx.buffer)?;
        {
            let inner = ctx.buffer.get_ref();
            hasher.update(&inner[..ctx.buffer.position() as usize]);
        }
        ctx.buffer.seek(SeekFrom::Start(0))?;

        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut ctx.buffer).ok();

        let date_time = match exif {
            Some(exif) => get_date_time(exif)?,
            None => None
        };
        ctx.buffer.seek(SeekFrom::Start(0))?;

        let mut file_hash = None;
        hasher.finalize_variable(|hash|{
            file_hash = Some(hex::encode(hash));
        });
        let file_hash = file_hash.unwrap();

        let mut dst_file_path = match date_time {
            Some(dt) => {
                let folder_rendered = dt.format(&folder_tpl).to_string();
                let file_rendered = format!("{}-{}", dt.format(&file_tpl).to_string(), file_hash);
                let mut path = dst_dir.join(PathBuf::from(folder_rendered));
                fs::create_dir_all(path.clone())?;
                path.push(file_rendered);
                path
            },
            None => {
                ctx.result.inc_unknown();
                unknown_path.join(file_hash)
            }
        };
        dst_file_path.set_extension(src_path.extension().unwrap());
        
        if !dst_file_path.exists() {
            let mut dst_file = File::create(dst_file_path)?;
            io::copy(&mut ctx.buffer, &mut dst_file)?;
            ctx.result.inc_copied();
        } else {
            ctx.result.inc_exist();
        }
   
        ctx.bar.inc(1);

        Ok(())
    })?;
    Ok(result)
}