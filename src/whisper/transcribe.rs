use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::config::AppConfig;
use crate::media::{extract_audio_from_video, generate_srt_file, parse_wav_file};
use std::env;
use std::fs;

/// 核心功能：从视频生成字幕
pub fn transcribe_audio(app_config: &AppConfig) -> Result<()> {
    let video_path = Path::new(&app_config.input);
    if !video_path.exists() {
        anyhow::bail!("视频文件不存在: {}", app_config.input);
    }

    // 确保模型存在，如果不存在则下载
    println!("检查模型: {}...", app_config.model);
    let model_path = crate::whisper::check_model_sync(&app_config.model)?;
    println!("使用模型: {:?}", model_path);

    let output_path = Path::new(&app_config.output);

    // 在系统临时目录创建临时WAV文件
    let temp_dir = env::temp_dir();
    let temp_audio_path = temp_dir.join("temp_audio.wav");

    println!("正在从视频中提取音频...");
    extract_audio_from_video(video_path, &temp_audio_path)?;

    println!("正在解析音频文件...");
    let original_samples = parse_wav_file(&temp_audio_path)?;

    let mut samples = vec![0.0f32; original_samples.len()];
    whisper_rs::convert_integer_to_float_audio(&original_samples, &mut samples)
        .context("无法转换音频样本")?;

    println!("正在加载Whisper模型...");
    let ctx = WhisperContext::new_with_params(
        model_path.to_string_lossy().as_ref(),
        WhisperContextParameters::default(),
    )
    .context("无法加载Whisper模型")?;

    println!("正在转录音频...");
    let mut state = ctx.create_state().context("无法创建状态")?;

    let mut params = FullParams::new(SamplingStrategy::default());
    if app_config.language != "auto" {
        params.set_language(Some(&app_config.language));
    }
    params.set_progress_callback_safe(|progress| println!("处理进度: {}%", progress));

    let start_time = std::time::Instant::now();

    state.full(params, &samples).context("转录失败")?;

    let elapsed = start_time.elapsed();
    println!("转录完成，耗时 {}ms", elapsed.as_millis());

    println!("正在生成字幕文件...");
    generate_srt_file(&state, output_path)?;

    // 清理临时文件
    if temp_audio_path.exists() {
        fs::remove_file(&temp_audio_path).context("无法删除临时音频文件")?;
    }

    println!("字幕生成完成: {}", app_config.output);

    Ok(())
}
