mod audio;
mod subtitle;

pub use audio::{extract_audio_from_video, parse_wav_file};
pub use subtitle::generate_srt_file;
