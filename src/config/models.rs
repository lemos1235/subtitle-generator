use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

// 默认模型名称
pub const DEFAULT_MODEL: &str = "ggml-medium-q8_0.bin";

#[derive(Debug, Deserialize)]
pub struct BaseConfig {
    pub model: String,
    pub language: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub base: BaseConfig,
}

pub struct AppConfig {
    pub input: String,
    pub output: String,
    pub model: String,
    pub language: String,
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            model: DEFAULT_MODEL.to_string(),
            language: "auto".to_string(),
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            base: BaseConfig::default(),
        }
    }
}

/// 获取配置文件路径
pub fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "video-subtitle")
        .context("无法确定用户配置目录")?;
    
    let config_dir = proj_dirs.config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(config_dir).context("无法创建配置目录")?;
    }
    
    Ok(config_dir.join("config.toml"))
}

/// 创建默认配置文件
fn create_default_config_file(path: &PathBuf) -> Result<()> {
    let default_config = format!(r#"# 视频字幕提取工具配置文件 - 自动生成

[base]
# Whisper模型名称
model = "{}"

# 识别语言 (例如: zh, ja, auto)
language = "auto"
"#, DEFAULT_MODEL);

    let mut file = fs::File::create(path).context(format!("无法创建配置文件: {:?}", path))?;
    file.write_all(default_config.as_bytes()).context("无法写入默认配置")?;
    println!("已创建默认配置文件: {:?}", path);
    
    Ok(())
}

/// 加载配置文件
pub fn load_config() -> Result<ConfigFile> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        println!("配置文件不存在，正在创建默认配置文件...");
        create_default_config_file(&config_path)?;
    }
    
    let mut file = fs::File::open(&config_path).context(format!("无法打开配置文件: {:?}", config_path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).context("无法读取配置文件内容")?;
    let config: ConfigFile = toml::from_str(&contents).context("无法解析TOML配置文件")?;
    Ok(config)
}
