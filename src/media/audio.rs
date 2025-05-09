use anyhow::{Context, Result};
use hound::{SampleFormat, WavReader};
use std::path::Path;
use std::process::Command;

/// 从视频中提取音频
pub fn extract_audio_from_video(video_path: &Path, audio_path: &Path) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().unwrap(),
            "-vn",
            "-ar",
            "16000",
            "-ac",
            "1",
            "-sample_fmt",
            "s16",
            "-f",
            "wav",
            audio_path.to_str().unwrap(),
        ])
        .output()
        .context("无法运行ffmpeg命令")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg命令失败: {}", error);
    }

    Ok(())
}

/// 解析WAV文件
pub fn parse_wav_file(path: &Path) -> Result<Vec<i16>> {
    let reader = WavReader::open(path).context("无法读取WAV文件")?;

    if reader.spec().channels != 1 {
        anyhow::bail!("期望单声道音频文件");
    }
    if reader.spec().sample_format != SampleFormat::Int {
        anyhow::bail!("期望整数样本格式");
    }
    if reader.spec().sample_rate != 16000 {
        anyhow::bail!("期望16KHz采样率");
    }
    if reader.spec().bits_per_sample != 16 {
        anyhow::bail!("期望16位每样本");
    }

    Ok(reader
        .into_samples::<i16>()
        .filter_map(Result::ok)
        .collect())
}
