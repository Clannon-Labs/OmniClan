
use std::path::Path;
use serde::Deserialize;
use std::io::{Error, ErrorKind};
use serde_json;
use tokio::process::Command;

use super::super::states::FrameRate;

#[derive(Debug, Deserialize)]
#[serde( rename_all = "snake_case" )]
pub(crate) struct ProbeOutput {
    pub(crate) streams: Vec<ProbeStream>,
    pub(crate) format: ProbeFormat,
}

#[derive(Debug, Deserialize)]
#[serde( rename_all = "snake_case" )]
pub(crate) struct ProbeFormat {
    pub(crate) filename: String,
    pub(crate) format_name: Option<String>,
    
    pub(crate) nb_streams: u32,
    
    pub(crate) duration: Option<String>,
    pub(crate) size: Option<String>,
    pub(crate) bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde( rename_all = "snake_case" )]
pub(crate) struct ProbeStream {
    pub(crate) index: i32,
    pub(crate) codec_name: String, // av1/mp3
    pub(crate) codec_type: String, // video/audio
    
    pub(crate) height: Option<u32>,
    pub(crate) width: Option<u32>,

    pub(crate) avg_frame_rate: Option<String>,
    
    pub(crate) sample_rate: Option<String>,
    pub(crate) channels: Option<u32>,
    pub(crate) duration: Option<String>,
    pub(crate) bit_rate: Option<String>,
    
    pub(crate) tags: Option<ProbeTags>,
}

#[derive(Debug, Deserialize)]
#[serde( rename_all = "snake_case" )]
pub(crate) struct ProbeTags {
    pub(crate) language: Option<String>,
}


pub(super) async fn probe_from_path(
    uploaded_media_path: &Path,
) -> Result<ProbeOutput, std::io::Error> {
    let output = match Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(uploaded_media_path)
        .output()
        .await {
            Ok(output) => output,
            Err(_) => {
                return Err(
                    std::io::Error::other(
                    "ffprobe failed!"
                ));
            }
        };

    let probe: ProbeOutput = serde_json::from_slice(&output.stdout)?;
    
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    if !output.status.success(){
        return Err(
            std::io::Error::other(
                format!("{stderr}")
            )
        );
    }

    if probe.streams.is_empty() {
        return Err(
            std::io::Error::other(
                "Stream is empty!"
            )
        );
    }

    Ok(probe)
}

impl ProbeOutput {
    pub(crate) fn has_video(&self) -> bool {
        self.
            streams
            .iter()
            .any(|s| s.codec_type == "video")
    }

    pub(crate) fn has_audio(&self) -> bool {
        self
            .streams
            .iter()
            .any(|s| s.codec_type == "audio")
    }
    
    pub(crate) fn video_streams(&self) -> Option<&ProbeStream> {
        self
            .streams
            .iter()
            .find(|s| s.codec_type == "video")
    }

    pub(crate) fn audio_streams(&self) -> impl Iterator<Item = &ProbeStream> {
        self
            .streams
            .iter()
            .filter(|s| s.codec_type == "audio")
    }

    pub(crate) fn duration_ms(&self) -> Result<u64, Error> {
        self.format.duration_ms()
    }
    
    pub(crate) fn size_bytes(&self) -> Result<u64, Error> {
        self.format.size_bytes()
    }
}

impl ProbeStream {
    pub(crate) fn index(&self) -> i32 {
        self.index
    }

    pub(crate) fn codec_name(&self) -> &str {
        &self.codec_name
    }

    pub(crate) fn codec_type(&self) -> &str {
        &self.codec_type
    }

    pub(crate) fn height(&self) -> Result<u32, Error> {
        self
            .height
            .ok_or_else(
                || Error::other(
                    "Couldn't find height!"
                )
            )
    }

    pub(crate) fn width(&self) -> Result<u32, Error> {
        self
            .width
            .ok_or_else(
                || Error::other(
                    "Couldn't find width!"
                )
            )
    }
    
    pub(crate) fn is_video(&self) -> bool {
        self.codec_type == "video"
    }

    pub(crate) fn is_audio(&self) -> bool {
        self.codec_type == "audio"
    }

    pub(crate) fn avg_fps(&self) -> Result<FrameRate, Error> {
        let (num, den) = self
            .avg_frame_rate
            .as_deref()
            .ok_or_else (
                || Error::other(
                    "Couldn't find frame rate!"
                )
            )
            .and_then(|rate|
                rate.split_once('/')
                    .ok_or_else(
                        || Error::other(
                            "Couldn't parse frame rate!"
                        )
                    )
            )?;

        let numerator: u32 = num
            .parse::<u32>()
            .map_err(
                |_| Error::new(
                    ErrorKind::InvalidData,
                    "Frame rate numerator not parsable!"
                )
            )?;

        let denominator: u32 = den
            .parse::<u32>()
            .map_err(
                |_| Error::new(
                    ErrorKind::InvalidData,
                    "Frame rate denominator not parsable!"
                )
            )?;

        if denominator == 0 {
            return Err(
                Error::new(
                    ErrorKind::InvalidData,
                    "Frame rate denominator is zero!"
                )
            )
        }

        Ok(FrameRate {
            numerator,
            denominator,
        })
    }
    
    pub(crate) fn sample_rate(&self) -> Result<u32, Error> {
        self
            .sample_rate
            .as_deref()
            .ok_or_else(
                || Error::other(
                    "No sample rate found!"
                )
            )?
            .parse::<u32>()
            .map_err(
                |_| Error::new(
                    ErrorKind::InvalidData,
                    "Couldn't parse sample rate!"
                )
            )
    }

    pub(crate) fn channels(&self) -> Result<u32, Error> {
        self
            .channels
            .ok_or_else(
                || Error::other(
                    "No channels found!"
                )
            ) 
    }

    // duration inside format is the better way to get the exact duration
    // of the media cuz individual streams are completely optional and
    // maybe missing
    // pub(crate) fn duration_ms(&self) -> Result<u64, Error> {
    //     let duration_s: f64 = self
    //         .duration
    //         .as_deref()
    //         .ok_or_else(
    //             || Error::other(
    //                 "No duration found!"
    //             )
    //         )?
    //         .parse::<f64>()
    //         .map_err(
    //             |_| Error::other(
    //                 "Couldn't parse duration!"
    //             )
    //         )?;

    //     let duration_ms: u64 = (duration_s * 1000.0) as u64;

    //     println!("[PROBE]: duration_ms() from ProbeStream returned {}", duration_ms);
        
    //     Ok(duration_ms)
    // }

    pub(crate) fn bit_rate(&self) -> Result<u32, Error> {
        self
            .bit_rate
            .as_deref()
            .ok_or_else(
                || Error::other(
                    "Couldn't find bit rate!"
                )
            )?
            .parse::<u32>()
            .map_err(
                |_| Error::other(
                    "Couldn't parse bit rate!"
                )
            )
    }

}

impl ProbeFormat {
    pub(crate) fn filename(&self) -> &str {
        &self.filename
    }
    
    pub(crate) fn format_name(&self) -> Result<&str, Error> {
        let name = &self
            .format_name
            .as_deref()
            .ok_or_else(
                || Error::other(
                    "Missing format name"
                )
            )
            .and_then(
                |name| name.split_once(",")
                    .ok_or_else(
                        || Error::other(
                            "Couldn't get the format names"
                        )
                    )
            )?;

        /*
         * Example: "matroska,webm"
         * What I am trying to get: webm
         */

        Ok(name.1)
    }

    pub(crate) fn nb_streams(&self) -> u32 {
        self.nb_streams
    }
    
    pub(crate) fn duration_ms(&self) -> Result<u64, Error> {
        let duration_s: f64 = self
            .duration
            .as_deref()
            .ok_or_else(
                || Error::other(
                    "Couldn't find duration!"
                )
            )?
            .parse::<f64>()
            .map_err(
                |_| Error::other(
                    "Couldn't parse duration!"
                )
            )?;

        // we take the size as float first to conserve
        // the milliseconds (eg: 127.123, 123 is milliseconds)
        // 
        // and later convert everything to milliseconds
        // instead of parsing directly in u64, this way
        // the floating part/milliseconds will be conserved
        let duration_ms = (duration_s * 1000.0) as u64;

        // println!("[PROBE]: duration_ms() from ProbeFormat returned {}", duration_ms);
        
        Ok(duration_ms)
    }

    pub(crate) fn size_bytes(&self) -> Result<u64, Error> {
        self
            .size
            .as_deref()
            .ok_or_else(
                || Error::other(
                    "Couldn't find size of the media!"
                )
            )?
            .parse::<u64>()
            .map_err(
                |_| Error::other(
                    "Couldn't parse size of the media!"
                )
            )
    }

    pub(crate) fn bit_rate(&self) -> Result<u32, Error> {
        self
            .bit_rate
            .as_deref()
            .ok_or_else(
                || Error::other(
                    "Couldn'd find bit rate!"
                )
            )?
            .parse::<u32>()
            .map_err(
                |_| Error::other(
                    "Couldn't parse the bit rate!"
                )
            )
    }
}

impl ProbeTags {
    pub(crate) fn language(&self) -> Option<&str> {
        self
            .language
            .as_deref()
    }
}
