// src/video.rs
use std::{fs::File, io::Read, io::Cursor, sync::mpsc::Sender, thread};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::get_probe;
use symphonia::core::codecs::CodecType;
use h264_decoder::Decoder as H264Decoder;
use anyhow::Result;

pub struct VideoFrame {
    pub image: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct VideoPlayer;

impl VideoPlayer {
    pub fn spawn(src: String, tx: Sender<VideoFrame>) -> Result<thread::JoinHandle<()>> {
        let handle = thread::spawn(move || { let _ = Self::run(src, tx); });
        Ok(handle)
    }

    fn run(src: String, tx: Sender<VideoFrame>) -> Result<()> {
        let mut data = Vec::new();
        if src.starts_with("http") {
            reqwest::blocking::get(&src)?.read_to_end(&mut data)?;
        } else {
            File::open(&src)?.read_to_end(&mut data)?;
        }

        let mss = MediaSourceStream::new(Box::new(Cursor::new(data)), Default::default());
        let hint = Hint::new();
        let probed = get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;
        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec == Some(CodecType::H264))
            .ok_or_else(|| anyhow::anyhow!("No H.264 track"))?;

        let mut decoder = H264Decoder::new();

        while let Ok(packet) = format.next_packet() {
            if packet.track_id() == track.id {
                let d = packet.data;
                let mut off = 0;
                while off + 4 <= d.len() {
                    let len = u32::from_be_bytes(d[off..off+4].try_into().unwrap()) as usize;
                    off += 4;
                    if off + len > d.len() { break; }
                    decoder.decode(&d[off..off+len])?;
                    off += len;
                }
                while let Some(frame) = decoder.get_frame() {
                    let w = frame.width() as usize;
                    let h = frame.height() as usize;
                    let y = frame.y_plane();
                    let u = frame.u_plane();
                    let v = frame.v_plane();
                    let mut rgba = vec![0u8; w*h*4];
                    for j in 0..h {
                        for i in 0..w {
                            let idx = j*w + i;
                            let uv = (j/2)*(w/2) + (i/2);
                            let yf = y[idx]  as f32;
                            let uf = u[uv] as f32 - 128.0;
                            let vf = v[uv] as f32 - 128.0;
                            let r = (yf + 1.402*vf).clamp(0.0,255.0) as u8;
                            let g = (yf - 0.34414*uf - 0.71414*vf).clamp(0.0,255.0) as u8;
                            let b = (yf + 1.772*uf).clamp(0.0,255.0) as u8;
                            let base = idx*4;
                            rgba[base  ] = r;
                            rgba[base+1] = g;
                            rgba[base+2] = b;
                            rgba[base+3] = 255;
                        }
                    }
                    tx.send(VideoFrame { image: rgba, width: w as u32, height: h as u32 })?;
                }
            }
        }

        Ok(())
    }
}
