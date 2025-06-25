use crate::audio::download_progress::show_download_progress;
use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::path::PathBuf;

// Configuration constants
const AUDIO_FILENAME: &str = "foregone_destruction_remastered.flac";
const AUDIO_URL: &str = "https://dn721905.ca.archive.org/0/items/unreal-tournament-ost-remastered/Unreal%20Tournament%20OST%20%28Remastered%29/10%20-%20Michiel%20van%20den%20Bos%20-%20Foregone%20Destruction%20%28Remastered%29.flac";
const OLD_AUDIO_FILES: &[&str] = &["shizuo_tribute_mix.flac", "botpack_9_michiel.mp3"];
// Expected file size range (approximately 50-80 MB for a high-quality FLAC file)
const MIN_EXPECTED_FILE_SIZE: u64 = 50_000_000;  // 50 MB
const MAX_EXPECTED_FILE_SIZE: u64 = 100_000_000; // 100 MB

pub async fn ensure_audio_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let audio_dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stimstation");

    let target_audio_path = audio_dir.join(AUDIO_FILENAME);

    // Check if the target file exists and is valid
    if target_audio_path.exists() && is_valid_audio_file(&target_audio_path)? {
        println!("Correct audio file found, loading...");
        return Ok(target_audio_path);
    }

    // Check for any old audio files and remove them
    if audio_dir.exists() {
        for old_file in OLD_AUDIO_FILES {
            let old_path = audio_dir.join(old_file);
            if old_path.exists() {
                println!("Removing old audio file: {}", old_file);
                std::fs::remove_file(old_path)?;
            }
        }
    }

    // Download the new file to a temporary location first
    let temp_path = target_audio_path.with_extension("tmp");
    println!("Starting audio file download with progress window...");
    show_download_progress(AUDIO_URL, &temp_path)?;

    // Verify the downloaded file
    if is_valid_audio_file(&temp_path)? {
        // Atomically move the temporary file to the final location
        std::fs::rename(&temp_path, &target_audio_path)?;
        println!("Audio file downloaded and verified successfully!");
    } else {
        // Clean up the invalid temporary file
        let _ = std::fs::remove_file(&temp_path);
        return Err("Downloaded file appears to be corrupted".into());
    }

    Ok(target_audio_path)
}

fn is_valid_audio_file(path: &std::path::Path) -> Result<bool, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(false);
    }

    // Check file size against expected range
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();
    if file_size < MIN_EXPECTED_FILE_SIZE || file_size > MAX_EXPECTED_FILE_SIZE {
        println!(
            "File size {} bytes is outside expected range ({} - {} bytes)",
            file_size, MIN_EXPECTED_FILE_SIZE, MAX_EXPECTED_FILE_SIZE
        );
        return Ok(false);
    }

    // Try to open the file with rodio to verify it's a valid audio file
    match std::fs::File::open(path) {
        Ok(file) => match Decoder::new(BufReader::new(file)) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        },
        Err(_) => Ok(false),
    }
}

pub fn setup_audio(
    audio_path: PathBuf,
) -> Result<(OutputStream, Sink), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let file = std::fs::File::open(audio_path)?;
    let source = Decoder::new(BufReader::new(file))?;
    sink.append(source);
    sink.play();

    Ok((_stream, sink))
}
