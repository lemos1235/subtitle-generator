use anyhow::{Context, Result};
use clap::Parser;
use subtitle_generator::config;
use subtitle_generator::whisper::transcribe_audio;

#[derive(Parser, Debug)]
#[command(author, version, about = "从视频中提取字幕")]
pub struct Args {
    /// 识别语言 (例如: zh, ja, auto)
    #[arg(short, long)]
    pub language: Option<String>,

    /// 输入视频文件路径
    pub input: Option<String>,

    /// 输出字幕文件路径
    pub output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = args.input.context("缺少输入视频文件路径")?;
    let output = args.output.context("缺少输出字幕文件路径")?;

    let config = config::load_config()?;
    let model = config.base.model;
    let language = args.language.unwrap_or(config.base.language);

    // 调用核心功能，直接传递参数
    transcribe_audio(&input, &output, &model, &language)
}
