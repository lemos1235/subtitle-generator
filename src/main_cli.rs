use anyhow::Result;
use clap::Parser;

use subtitle_generator::cli::{Args, args::create_app_config};
use subtitle_generator::whisper::transcribe_audio;

fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();
    
    // 创建应用配置
    let app_config = create_app_config(args)?;
    
    // 调用核心功能
    transcribe_audio(&app_config)
}
