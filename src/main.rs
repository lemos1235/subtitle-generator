use anyhow::{Context, Result};
use clap::Parser;
use subtitle_generator::whisper::transcribe_audio;
use subtitle_generator::{config, AppConfig};

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
    // 解析命令行参数
    let args = Args::parse();

    // 创建应用配置
    let input = args.input.context("缺少输入视频文件路径")?;
    let output = args.output.context("缺少输出字幕文件路径")?;

    // 加载配置文件
    let config = config::load_config()?;

    // 创建应用配置
    let app_config = AppConfig {
        input,
        output,
        model: config.base.model,
        language: args.language.unwrap_or(config.base.language),
    };

    // 调用核心功能
    transcribe_audio(&app_config)
}
