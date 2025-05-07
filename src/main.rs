use anyhow::{Context, Result};
use clap::Parser;
use hound::{SampleFormat, WavReader};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use video_subtitle::{check_model_sync, AppConfig};

#[derive(Parser, Debug)]
#[command(author, version, about = "从视频中提取字幕")]
struct Args {
    /// 识别语言 (例如: zh, ja, auto)
    #[arg(short, long)]
    language: Option<String>,

    /// 输入视频文件路径
    input: Option<String>,

    /// 输出字幕文件路径
    output: Option<String>,
}

/// 从视频中提取音频
fn extract_audio_from_video(video_path: &Path, audio_path: &Path) -> Result<()> {
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
fn parse_wav_file(path: &Path) -> Result<Vec<i16>> {
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

/// 格式化时间戳
fn format_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let millisecs = ((seconds % 1.0) * 1000.0) as u32;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millisecs)
}

/// 生成SRT格式字幕
fn generate_srt_file(state: &whisper_rs::WhisperState, output_path: &Path) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

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

/// 核心功能：从视频生成字幕
fn generate_subtitles(app_config: &AppConfig) -> Result<()> {
    let video_path = Path::new(&app_config.input);
    if !video_path.exists() {
        anyhow::bail!("视频文件不存在: {}", app_config.input);
    }

    // 确保模型存在，如果不存在则下载
    println!("检查模型: {}...", app_config.model);
    let model_path = check_model_sync(&app_config.model)?;
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

fn main() -> Result<()> {
    let args = Args::parse();

    let input = args.input.context("缺少输入视频文件路径")?;
    let output = args.output.context("缺少输出字幕文件路径")?;
    let language = args.language;

    // 创建应用配置
    let config = video_subtitle::config::load_config()?;
    let app_config = AppConfig {
        input,
        output,
        model: config.base.model,
        language: language.unwrap_or(config.base.language),
    };

    // 调用核心功能
    generate_subtitles(&app_config)
}
