use anyhow::Result;
use eframe::{egui, CreationContext};
use egui::{CentralPanel, Color32, ProgressBar, RichText, TopBottomPanel, Ui};
use rfd::FileDialog;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::{load_config, AppConfig};
use crate::whisper::transcribe_audio;

/// 应用状态
#[derive(Clone, PartialEq)]
enum AppStatus {
    Ready,         // 准备就绪
    Processing,    // 处理中
    Completed,     // 完成
    Error(String), // 错误
}

/// 进度信息
struct ProgressInfo {
    status: AppStatus,
    message: String,
    progress: f32,
}

impl Default for ProgressInfo {
    fn default() -> Self {
        Self {
            status: AppStatus::Ready,
            message: "准备就绪".to_string(),
            progress: 0.0,
        }
    }
}

/// 视频字幕应用
pub struct VideoSubtitleApp {
    input_path: Option<String>,
    output_path: Option<String>,
    language: String,
    model: String,
    progress_info: Arc<Mutex<ProgressInfo>>,
}

impl Default for VideoSubtitleApp {
    fn default() -> Self {
        // 加载默认配置
        let config = load_config().unwrap_or_else(|_| {
            eprintln!("无法加载配置，使用默认值");
            crate::config::ConfigFile::default()
        });

        Self {
            input_path: None,
            output_path: None,
            language: config.base.language,
            model: config.base.model,
            progress_info: Arc::new(Mutex::new(ProgressInfo::default())),
        }
    }
}

impl VideoSubtitleApp {
    /// 创建新应用
    pub fn new(cc: &CreationContext) -> Self {
        // 设置默认样式
        setup_custom_styles(&cc.egui_ctx);
        Self::default()
    }

    /// 选择输入文件
    fn select_input_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("视频文件", &["mp4", "avi", "mov", "mkv"])
            .set_title("选择视频文件")
            .pick_file()
        {
            self.input_path = Some(path.to_string_lossy().to_string());

            // 自动生成输出路径
            if self.output_path.is_none() {
                let output = format!("{}.srt", path.file_stem().unwrap().to_string_lossy());
                let output_path = path.with_file_name(output);
                self.output_path = Some(output_path.to_string_lossy().to_string());
            }
        }
    }

    /// 选择输出文件
    fn select_output_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("字幕文件", &["srt"])
            .set_title("保存字幕文件")
            .set_file_name("output.srt")
            .save_file()
        {
            self.output_path = Some(path.to_string_lossy().to_string());
        }
    }

    /// 开始处理
    fn start_processing(&mut self) {
        if self.input_path.is_none() || self.output_path.is_none() {
            let mut info = self.progress_info.lock().unwrap();
            info.status = AppStatus::Error("请先选择输入和输出文件".to_string());
            return;
        }

        let input_path = self.input_path.clone().unwrap();
        let output_path = self.output_path.clone().unwrap();
        let language = self.language.clone();
        let model = self.model.clone();

        let progress_info = self.progress_info.clone();

        // 更新状态为处理中
        {
            let mut info = progress_info.lock().unwrap();
            info.status = AppStatus::Processing;
            info.message = "正在准备...".to_string();
            info.progress = 0.0;
        }

        // 创建配置
        let app_config = AppConfig {
            input: input_path,
            output: output_path,
            model,
            language,
        };

        // 在新线程中处理，避免阻塞UI
        thread::spawn(move || {
            // 处理过程中的进度回调
            let progress_callback = {
                let progress_info = progress_info.clone();
                move |msg: &str, progress: f32| {
                    let mut info = progress_info.lock().unwrap();
                    info.message = msg.to_string();
                    info.progress = progress;
                }
            };

            // 调用核心处理功能
            match process_with_progress(&app_config, progress_callback) {
                Ok(_) => {
                    let mut info = progress_info.lock().unwrap();
                    info.status = AppStatus::Completed;
                    info.message = "处理完成！".to_string();
                    info.progress = 1.0;
                }
                Err(e) => {
                    let mut info = progress_info.lock().unwrap();
                    info.status = AppStatus::Error(format!("错误: {}", e));
                    info.progress = 0.0;
                }
            }
        });
    }

    /// 渲染顶部面板
    fn render_top_panel(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("视频字幕提取工具");
        });
    }

    /// 渲染底部状态栏
    fn render_bottom_panel(&self, ui: &mut Ui) {
        let info = self.progress_info.lock().unwrap();

        match &info.status {
            AppStatus::Ready => {
                ui.label("准备就绪");
            }
            AppStatus::Processing => {
                ui.horizontal(|ui| {
                    ui.label(&info.message);
                    ui.add(ProgressBar::new(info.progress).animate(true));
                });
            }
            AppStatus::Completed => {
                ui.colored_label(Color32::GREEN, "处理完成！");
            }
            AppStatus::Error(err) => {
                ui.colored_label(Color32::RED, err);
            }
        }
    }

    /// 渲染主要内容区域
    fn render_main_panel(&mut self, ui: &mut Ui) {
        let processing = matches!(
            self.progress_info.lock().unwrap().status,
            AppStatus::Processing
        );

        // 文件选择区域
        ui.group(|ui| {
            ui.heading("文件选择");
            ui.horizontal(|ui| {
                ui.label("输入视频:");
                if let Some(input) = &self.input_path {
                    let file_name = Path::new(input)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    ui.label(format!("{}", file_name));
                } else {
                    ui.label("未选择");
                }
                if ui.button("浏览...").clicked() && !processing {
                    self.select_input_file();
                }
            });

            ui.horizontal(|ui| {
                ui.label("输出字幕:");
                if let Some(output) = &self.output_path {
                    let file_name = Path::new(output)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    ui.label(format!("{}", file_name));
                } else {
                    ui.label("未选择");
                }
                if ui.button("浏览...").clicked() && !processing {
                    self.select_output_file();
                }
            });
        });

        // 参数设置区域
        ui.group(|ui| {
            ui.heading("参数设置");
            ui.horizontal(|ui| {
                ui.label("语言:");
                ui.add_enabled_ui(!processing, |ui| {
                    egui::ComboBox::from_id_source("language")
                        .selected_text(&self.language)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.language, "auto".to_string(), "自动检测");
                            ui.selectable_value(&mut self.language, "zh".to_string(), "中文");
                            ui.selectable_value(&mut self.language, "en".to_string(), "英语");
                            ui.selectable_value(&mut self.language, "ja".to_string(), "日语");
                        });
                });
            });

            ui.horizontal(|ui| {
                ui.label("模型:");
                ui.text_edit_singleline(&mut self.model);
            });
        });

        // 操作按钮区域
        ui.vertical_centered(|ui| {
            let button_text = match self.progress_info.lock().unwrap().status {
                AppStatus::Processing => "处理中...",
                AppStatus::Completed => "重新处理",
                _ => "开始处理",
            };

            let button = egui::Button::new(RichText::new(button_text).size(18.0));
            if ui.add_enabled(!processing, button).clicked() {
                self.start_processing();
            }
        });
    }
}

impl eframe::App for VideoSubtitleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置UI刷新率
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // 顶部面板
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_top_panel(ui);
        });

        // 底部状态栏
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.render_bottom_panel(ui);
        });

        // 主面板
        CentralPanel::default().show(ctx, |ui| {
            self.render_main_panel(ui);
        });
    }
}

/// 设置自定义样式
fn setup_custom_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);

    // 配置字体
    let mut fonts = egui::FontDefinitions::default();

    if let Ok(font_data) = std::fs::read("assets/fonts/NotoSansSC-Regular.ttf") {
        fonts.font_data.insert(
            "chinese_font".to_owned(),
            egui::FontData::from_owned(font_data),
        );
        // 将中文字体添加到所有字体族中
        for (_family_name, family) in fonts.families.iter_mut() {
            family.insert(0, "chinese_font".to_owned());
        }
    }

    // 应用字体设置
    ctx.set_fonts(fonts);
    ctx.set_style(style);
}

/// 带进度回调的处理函数封装
fn process_with_progress<F>(config: &AppConfig, progress_callback: F) -> Result<()>
where
    F: Fn(&str, f32) + Send + 'static,
{
    // TODO: 这里应该修改transcribe_audio函数以支持进度回调
    // 目前直接调用原函数，后续可以改进
    progress_callback("正在处理视频...", 0.5);
    transcribe_audio(config)?;
    progress_callback("处理完成", 1.0);
    Ok(())
}
