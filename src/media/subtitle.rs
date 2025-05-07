use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use whisper_rs::WhisperState;

/// 格式化时间戳
pub fn format_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let millisecs = ((seconds % 1.0) * 1000.0) as u32;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millisecs)
}

/// 生成SRT格式字幕
pub fn generate_srt_file(state: &WhisperState, output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path).context("无法创建字幕文件")?;

    let num_segments = state.full_n_segments().context("无法获取段落数量")?;

    for i in 0..num_segments {
        let segment_text = state.full_get_segment_text(i).context("无法获取段落文本")?;

        let start_time = state.full_get_segment_t0(i).context("无法获取开始时间戳")?;

        let end_time = state.full_get_segment_t1(i).context("无法获取结束时间戳")?;

        let start_formatted = format_timestamp(start_time as f64 / 100.0);
        let end_formatted = format_timestamp(end_time as f64 / 100.0);

        writeln!(file, "{}", i + 1)?;
        writeln!(file, "{} --> {}", start_formatted, end_formatted)?;
        writeln!(file, "{}", segment_text.trim())?;
        writeln!(file)?;
    }

    Ok(())
}
