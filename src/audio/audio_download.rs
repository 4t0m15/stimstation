use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::path::PathBuf;
use crate::audio::download_progress::show_download_progress;

pub async fn ensure_audio_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let audio_path = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stimstation")
        .join("shizuo_tribute_mix.flac");
    
    if !audio_path.exists() {
        println!("Starting audio file download with progress window...");
        
        // Use the progress window for download
        let url = "https://ia903409.us.archive.org/24/items/noistruct-shizuo-tribute-mix/noistruct%20-%20shizuo%20tribute%20mix.flac".to_string();
        show_download_progress(url, audio_path.clone())?;
        
        println!("Audio file downloaded successfully!");
    } else {
        println!("Audio file found, loading...");
    }
    
    Ok(audio_path)
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


