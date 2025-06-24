use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::path::PathBuf;
use crate::audio::download_progress::show_download_progress;

pub async fn ensure_audio_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let audio_dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stimstation");
    
    let target_audio_path = audio_dir.join("foregone_destruction_remastered.flac");
    let expected_url = "https://dn721905.ca.archive.org/0/items/unreal-tournament-ost-remastered/Unreal%20Tournament%20OST%20%28Remastered%29/10%20-%20Michiel%20van%20den%20Bos%20-%20Foregone%20Destruction%20%28Remastered%29.flac";
    
    // Check if the target file exists and is valid
    if target_audio_path.exists() && is_valid_audio_file(&target_audio_path)? {
        println!("Correct audio file found, loading...");
        return Ok(target_audio_path);
    }
    
    // Check for any old audio files and remove them
    if audio_dir.exists() {
        let old_files = ["shizuo_tribute_mix.flac", "botpack_9_michiel.mp3"];
        for old_file in &old_files {
            let old_path = audio_dir.join(old_file);
            if old_path.exists() {
                println!("Removing old audio file: {}", old_file);
                std::fs::remove_file(old_path)?;
            }
        }
    }
    
    // Download the new file
    println!("Starting audio file download with progress window...");
    show_download_progress(expected_url.to_string(), target_audio_path.clone())?;
    
    // Verify the downloaded file
    if is_valid_audio_file(&target_audio_path)? {
        println!("Audio file downloaded and verified successfully!");
    } else {
        return Err("Downloaded file appears to be corrupted".into());
    }
    
    Ok(target_audio_path)
}

fn is_valid_audio_file(path: &std::path::Path) -> Result<bool, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(false);
    }
    
    // Check file size (should be > 1MB for a music file)
    let metadata = std::fs::metadata(path)?;
    if metadata.len() < 1_000_000 {
        return Ok(false);
    }
    
    // Try to open the file with rodio to verify it's a valid audio file
    match std::fs::File::open(path) {
        Ok(file) => {
            match Decoder::new(BufReader::new(file)) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}

pub fn setup_audio(audio_path: PathBuf) -> Result<(OutputStream, Sink), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    let file = std::fs::File::open(audio_path)?;
    let source = Decoder::new(BufReader::new(file))?;
    sink.append(source);
    sink.play();
    
    Ok((_stream, sink))
}


