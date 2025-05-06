use anyhow::{Context, Result};
use directories::ProjectDirs;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

/// 获取模型目录路径
pub fn get_models_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "video-subtitle")
        .context("无法确定用户配置目录")?;
    
    let config_dir = proj_dirs.config_dir();
    let models_dir = config_dir.join("models");
    
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir).context("无法创建models目录")?;
    }
    
    Ok(models_dir)
}

/// 检查模型文件是否存在，如果不存在则下载
pub async fn ensure_model_exists(model_name: &str) -> Result<PathBuf> {
    let models_dir = get_models_dir()?;
    let model_path = models_dir.join(model_name);
    
    if !model_path.exists() {
        println!("模型文件 {} 不存在，开始下载...", model_name);
        download_model(model_name, &model_path).await?;
    }

    Ok(model_path)
}

/// 下载模型文件
async fn download_model(model_name: &str, model_path: &Path) -> Result<()> {
    let url = format!("{}/{}", MODEL_BASE_URL, model_name);
    
    println!("从 {} 下载模型...", url);
    
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .context("请求下载模型失败")?;
    
    let total_size = res
        .content_length()
        .context("无法获取模型文件大小")?;
    
    // 创建进度条
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .context("无法设置进度条样式")?
        .progress_chars("#>-"));
    
    // 下载文件
    let mut file = fs::File::create(model_path).context("无法创建模型文件")?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    
    while let Some(item) = stream.next().await {
        let chunk = item.context("下载过程中发生错误")?;
        file.write_all(&chunk).context("写入模型文件失败")?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }
    
    pb.finish_with_message(format!("下载完成: {}", model_name));
    
    Ok(())
}

/// 获取模型的完整路径
pub fn get_model_path(model_name: &str) -> Result<PathBuf> {
    let models_dir = get_models_dir()?;
    Ok(models_dir.join(model_name))
}

/// 同步版本的确保模型存在
pub fn ensure_model_exists_sync(model_name: &str) -> Result<PathBuf> {
    let rt = tokio::runtime::Runtime::new().context("无法创建Tokio运行时")?;
    rt.block_on(ensure_model_exists(model_name))
}
