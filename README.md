# 视频字幕提取工具

这是一个使用Rust编写的命令行工具，可以从视频文件中提取音频并生成SRT格式的字幕文件。该工具使用Whisper语音识别模型进行音频转写。

## 依赖项

- Rust 和 Cargo
- FFmpeg (需要在系统中安装并添加到PATH)

## 编译安装

```bash
# 克隆此仓库
git clone https://github.com/yourusername/video-subtitle.git
cd video-subtitle

# 编译项目
cargo build --release
```

## 使用方法

```bash
# 基本用法
video-subtitle input_video.mp4 output_subtitles.srt

# 指定语言
video-subtitle -l zh input_video.mp4 output_subtitles.srt
```

## 配置文件

程序使用位于系统用户配置目录的配置文件：

- Windows: `%APPDATA%\video-subtitle\config.toml`
- macOS: `~/Library/Application Support/video-subtitle/config.toml`
- Linux: `~/.config/video-subtitle/config.toml`

配置文件使用TOML格式：

```toml
[base]
# Whisper模型名称
model = "ggml-medium-q8_0.bin"

# 识别语言 (例如: zh, ja, auto)
language = "auto"
```

**注意**：
- language 参数可以通过命令行 `-l` 选项覆盖
- 模型路径只能通过配置文件设置

## Whisper模型

程序使用 Whisper 模型进行语音识别。默认使用 `ggml-medium-q8_0.bin` 模型。

**模型存储位置**：
- Windows: `%APPDATA%\video-subtitle\models\`
- macOS: `~/Library/Application\ Support/video-subtitle/models/`
- Linux: `~/.config/video-subtitle/models/`

**自动下载功能**：
- 首次运行时，如果模型不存在，程序会自动从Hugging Face下载所需的模型
- 下载的模型将保存在用户配置目录的 `models` 目录中

### 可用模型列表

所有模型均从 [ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) 下载。可用模型包括：

| 模型名称                | 大小      | 说明                                     |
| ----------------------- | --------- | ---------------------------------------- |
| tiny.bin                | 75 MB     | 超小型模型                              |
| tiny-q5_1.bin           | 31 MB     | 超小型模型 (5位量化版本)                |
| tiny-q8_0.bin           | 42 MB     | 超小型模型 (8位量化版本)                |
| tiny.en.bin             | 75 MB     | 超小型模型 (英文专用)                    |
| base.bin                | 142 MB    | 基础模型                                 |
| base-q5_1.bin           | 57 MB     | 基础模型 (5位量化版本)                  |
| base-q8_0.bin           | 78 MB     | 基础模型 (8位量化版本)                  |
| base.en.bin             | 142 MB    | 基础模型 (英文专用)                      |
| small.bin               | 466 MB    | 小型模型                                 |
| small-q5_1.bin          | 181 MB    | 小型模型 (5位量化版本)                  |
| small-q8_0.bin          | 252 MB    | 小型模型 (8位量化版本)                  |
| small.en.bin            | 466 MB    | 小型模型 (英文专用)                      |
| medium.bin              | 1.5 GB    | 中型模型                                 |
| medium-q5_0.bin         | 514 MB    | 中型模型 (5位量化版本)                  |
| medium-q8_0.bin         | 785 MB    | 中型模型 (8位量化版本)，**默认模型**    |
| medium.en.bin           | 1.5 GB    | 中型模型 (英文专用)                      |
| large-v1.bin            | 2.9 GB    | 大型模型 (版本1)                        |
| large-v2.bin            | 2.9 GB    | 大型模型 (版本2)                        |
| large-v2-q5_0.bin       | 1.1 GB    | 大型模型v2 (5位量化版本)                |
| large-v2-q8_0.bin       | 1.5 GB    | 大型模型v2 (8位量化版本)                |
| large-v3.bin            | 2.9 GB    | 大型模型 (版本3)                        |
| large-v3-q5_0.bin       | 1.1 GB    | 大型模型v3 (5位量化版本)                |
| large-v3-turbo.bin      | 1.5 GB    | 大型模型v3 turbo版                      |
| large-v3-turbo-q5_0.bin | 547 MB    | 大型模型v3 turbo (5位量化版本)          |
| large-v3-turbo-q8_0.bin | 834 MB    | 大型模型v3 turbo (8位量化版本)          |

**关于量化版本**：
- q5_0/q5_1: 5位量化，更小的文件大小，略有精度损失，速度更快
- q8_0: 8位量化，平衡文件大小和精度，速度适中
