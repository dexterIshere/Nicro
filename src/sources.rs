use std::{collections::HashMap, fs};

use songbird::input::{self, cached::Memory};

use crate::CachedSound;

pub async fn init_sources() -> Result<HashMap<String, CachedSound>, Box<dyn std::error::Error>> {
    let mut audio_map = HashMap::new();
    //  let mut audio_join_map = HashMap::new();

    let entries = fs::read_dir("static/sounds/")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_stem() {
                if let Some(file_name_str) = file_name.to_str() {
                    let src = Memory::new(
                        input::ffmpeg(path.to_str().ok_or("Failed to convert path to str")?)
                            .await?,
                    )?;

                    let _ = src.raw.spawn_loader();

                    audio_map.insert(file_name_str.to_owned(), CachedSound::Uncompressed(src));
                }
            }
        }
    }

    Ok(audio_map)
}

pub async fn init_sb_sources() -> Result<HashMap<String, CachedSound>, Box<dyn std::error::Error>> {
    let mut sb_sources_box = HashMap::new();
    let sb_entries = fs::read_dir("static/signed/")?;

    for entry in sb_entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_stem() {
                if let Some(file_name_str) = file_name.to_str() {
                    let src = Memory::new(
                        input::ffmpeg(path.to_str().ok_or("Failed to convert path to str")?)
                            .await?,
                    )?;

                    let _ = src.raw.spawn_loader();

                    sb_sources_box.insert(file_name_str.to_owned(), CachedSound::Uncompressed(src));
                }
            }
        }
    }

    Ok(sb_sources_box)
}

pub fn _get_sources_data() {}
