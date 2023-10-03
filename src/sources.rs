use std::collections::HashMap;

use songbird::input::{self, cached::Memory};
use tokio::fs;

use crate::CachedSound;

pub async fn init_sources() -> Result<HashMap<String, CachedSound>, Box<dyn std::error::Error>> {
    let mut audio_map = HashMap::new();
    //  let mut audio_join_map = HashMap::new();

    let mut entries = fs::read_dir("static/").await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        println!("{}", path.display());

        if path.is_file() {
            let file_name = path.file_stem().ok_or("Failed to read file name")?;
            let file_name_str = file_name.to_str().ok_or("Failed to convert OsStr to str")?;
            println!("{}", file_name_str);

            let src = Memory::new(
                input::ffmpeg(path.to_str().ok_or("Failed to convert path to str")?).await?,
            )?;

            let _ = src.raw.spawn_loader();

            audio_map.insert(file_name_str.to_string(), CachedSound::Uncompressed(src));
        }
    }

    Ok(audio_map)
}
