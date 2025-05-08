#!/bin/bash
# 这个脚本用于下载ffmpeg并将其放入应用程序包的正确位置

set -e

# 创建目录
mkdir -p assets/bin

# 下载FFmpeg for macOS (使用Helmut K. C. Tessarek提供的静态构建版本)
FFMPEG_URL="https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip"
FFPROBE_URL="https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip"

echo "下载FFmpeg..."
curl -sL $FFMPEG_URL -o /tmp/ffmpeg.zip
echo "下载FFprobe..."
curl -sL $FFPROBE_URL -o /tmp/ffprobe.zip

# 解压
echo "解压文件..."
unzip -o /tmp/ffmpeg.zip -d /tmp
unzip -o /tmp/ffprobe.zip -d /tmp

# 移动到正确位置
echo "拷贝到应用资源目录..."
cp /tmp/ffmpeg assets/bin/
cp /tmp/ffprobe assets/bin/

# 设置可执行权限
chmod +x assets/bin/ffmpeg
chmod +x assets/bin/ffprobe

echo "完成! FFmpeg已准备就绪。" 