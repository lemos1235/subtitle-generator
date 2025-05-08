#!/bin/bash
# 这个脚本用于构建和打包macOS应用程序

set -e

echo "准备FFmpeg..."
./scripts/prepare_ffmpeg.sh

echo "构建Release版本..."
cargo build --release --bin subtitle-generator-gui

echo "打包macOS应用程序..."
# 使用cargo-bundle打包应用程序 (需要先安装cargo-bundle)
# cargo install cargo-bundle
cargo bundle --release

echo "应用程序打包完成!"
echo "应用程序位于: target/release/bundle/osx/Subtitle Generator.app" 