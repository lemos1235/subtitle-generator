use anyhow::{Context, Result};
use clap::Parser;

use crate::config;
use crate::config::AppConfig;

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

/// 从命令行参数创建应用配置
pub fn create_app_config(args: Args) -> Result<AppConfig> {
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

    Ok(app_config)
}
