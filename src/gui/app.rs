use anyhow::Result;
use eframe::{egui, CreationContext};
use egui::{CentralPanel, Color32, RichText, Rounding, TopBottomPanel, Ui};
use rfd::FileDialog;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::{load_config, AppConfig};
use crate::whisper::transcribe_audio;

/// 应用状态
#[derive(Clone, PartialEq)]
enum AppStatus {
    Initial,       // 初始状态，等待选择文件
    FileSelected,  // 已选择文件，等待处理
    Processing,    // 处理中
    Completed,     // 完成，等待保存
    SaveSuccess,   // 保存成功
    Error(String), // 错误
}

/// 进度信息
struct ProgressInfo {
    status: AppStatus,
    message: String,
    progress: f32,
    subtitle_content: Option<String>, // 保存生成的字幕内容
}

impl Default for ProgressInfo {
    fn default() -> Self {
        Self {
            status: AppStatus::Initial,
            message: "请选择视频文件".to_string(),
            progress: 0.0,
            subtitle_content: None,
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

    /// 创建统一样式的按钮
    fn create_button<'a>(&self, ui: &'a mut Ui, text: &str) -> egui::Response {
        let button_size = egui::vec2(80.0, 26.0);
        ui.add(
            egui::Button::new(RichText::new(text).size(13.5).color(Color32::WHITE))
                .fill(Color32::from_rgb(0x13, 0x7A, 0x50))
                .rounding(Rounding::same(4.0))
                .min_size(button_size),
        )
    }

    /// 选择输入文件
    fn select_input_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("视频文件", &["mp4", "avi", "mov", "mkv"])
            .set_title("选择视频文件")
            .pick_file()
        {
            self.input_path = Some(path.to_string_lossy().to_string());

            // 更新状态为已选择文件
            let mut info = self.progress_info.lock().unwrap();
            info.status = AppStatus::FileSelected;
            info.message = "文件已选择，请点击生成字幕".to_string();
        } else {
            // 如果取消选择，则回到初始状态
            if self.progress_info.lock().unwrap().status == AppStatus::FileSelected {
                self.reset_to_initial();
            }
        }
    }

    /// 保存字幕文件
    fn save_subtitle_file(&mut self) {
        let content = {
            let info = self.progress_info.lock().unwrap();
            if let Some(ref content) = info.subtitle_content {
                content.clone()
            } else {
                return;
            }
        };

        // 根据输入文件名生成默认输出文件名
        let default_filename = if let Some(input) = &self.input_path {
            let path = Path::new(input);
            format!(
                "{}.srt",
                path.file_stem().unwrap_or_default().to_string_lossy()
            )
        } else {
            "output.srt".to_string()
        };

        if let Some(path) = FileDialog::new()
            .add_filter("字幕文件", &["srt"])
            .set_title("保存字幕文件")
            .set_file_name(&default_filename)
            .save_file()
        {
            self.output_path = Some(path.to_string_lossy().to_string());

            // 保存字幕内容到文件
            if let Err(e) = std::fs::write(&path, content) {
                let mut info = self.progress_info.lock().unwrap();
                info.status = AppStatus::Error(format!("保存文件失败: {}", e));
            } else {
                let mut info = self.progress_info.lock().unwrap();
                info.status = AppStatus::SaveSuccess;
                info.message = "字幕已保存成功！".to_string();
            }
        } else {
            // 如果取消保存，回到初始状态
            self.reset_to_initial();
        }
    }

    /// 重置为初始状态
    fn reset_to_initial(&mut self) {
        self.input_path = None;
        self.output_path = None;

        let mut info = self.progress_info.lock().unwrap();
        info.status = AppStatus::Initial;
        info.message = "请选择视频文件".to_string();
        info.progress = 0.0;
        info.subtitle_content = None;
    }

    /// 开始处理
    fn start_processing(&mut self) {
        if self.input_path.is_none() {
            let mut info = self.progress_info.lock().unwrap();
            info.status = AppStatus::Error("请先选择输入文件".to_string());
            return;
        }

        let input_path = self.input_path.clone().unwrap();
        // 生成临时输出路径
        let temp_output_path = format!("{}.temp.srt", input_path);
        let language = self.language.clone();
        let model = self.model.clone();

        let progress_info = self.progress_info.clone();

        // 更新状态为处理中
        {
            let mut info = progress_info.lock().unwrap();
            info.status = AppStatus::Processing;
            info.message = "正在生成字幕...".to_string();
            info.progress = 0.0;
        }

        // 创建配置
        let app_config = AppConfig {
            input: input_path,
            output: temp_output_path.clone(),
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
                    // 读取生成的字幕内容
                    match std::fs::read_to_string(&temp_output_path) {
                        Ok(content) => {
                            // 保存字幕内容到共享变量
                            let mut info = progress_info.lock().unwrap();
                            info.subtitle_content = Some(content);
                            info.status = AppStatus::Completed;
                            info.message = "字幕生成完成！".to_string();
                            info.progress = 1.0;

                            // 删除临时文件
                            if let Err(e) = std::fs::remove_file(&temp_output_path) {
                                eprintln!("无法删除临时文件: {}", e);
                            }
                        }
                        Err(e) => {
                            let mut info = progress_info.lock().unwrap();
                            info.status = AppStatus::Error(format!("读取生成的字幕失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let mut info = progress_info.lock().unwrap();
                    info.status = AppStatus::Error(format!("错误: {}", e));
                    info.progress = 0.0;
                }
            }
        });
    }

    /// 渲染底部状态栏
    fn render_bottom_panel(&self, ui: &mut Ui) {
        let info = self.progress_info.lock().unwrap();

        match &info.status {
            AppStatus::Initial => {
                ui.label(RichText::new("请选择视频文件").size(12.0));
            }
            AppStatus::FileSelected => {
                ui.label(RichText::new("请生成字幕").size(12.0));
            }
            AppStatus::Processing => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("执行中..").size(12.0));
                });
            }
            AppStatus::Completed => {
                ui.label(RichText::new("字幕生成完成！请保存文件").size(12.0));
            }
            AppStatus::SaveSuccess => {
                ui.label(RichText::new("保存成功").size(12.0));
            }
            AppStatus::Error(err) => {
                ui.colored_label(Color32::RED, RichText::new(err).size(12.0));
            }
        }
    }

    /// 渲染主要内容区域
    fn render_main_panel(&mut self, ui: &mut Ui) {
        let status = self.progress_info.lock().unwrap().status.clone();

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                // 使用居中布局
                ui.vertical_centered(|ui| {
                    // 创建一个固定宽度的区域，避免内容被拉伸
                    let available_height = ui.available_height();
                    ui.allocate_ui_with_layout(
                        egui::vec2(300.0, available_height),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            // 添加一些垂直空间，使内容垂直居中
                            let content_height = match status {
                                AppStatus::Initial => 28.0,       // 单个按钮的高度
                                AppStatus::FileSelected => 140.0, // 文本和两个按钮的高度
                                AppStatus::Processing => 22.0,    // 单个文本的高度
                                AppStatus::Completed => 80.0,     // 文本和一个按钮的高度
                                AppStatus::SaveSuccess => 80.0,   // 文本和一个按钮的高度
                                AppStatus::Error(_) => 80.0,      // 文本和一个按钮的高度
                            };

                            let top_margin = (available_height - content_height) / 2.0;
                            if top_margin > 0.0 {
                                ui.add_space(top_margin);
                            }

                            match status {
                                AppStatus::Initial => {
                                    // 初始界面只显示"选择文件"按钮
                                    if self.create_button(ui, "选择文件").clicked() {
                                        self.select_input_file();
                                    }
                                }

                                AppStatus::FileSelected => {
                                    // 显示已选择的文件和操作按钮
                                    if let Some(input) = &self.input_path {
                                        let file_name = Path::new(input)
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy();
                                        ui.label(
                                            RichText::new(format!("已选择: {}", file_name))
                                                .size(14.0),
                                        );
                                        ui.add_space(13.0);
                                    }

                                    if self.create_button(ui, "重新选择").clicked() {
                                        self.select_input_file();
                                    }

                                    ui.add_space(4.0);

                                    if self.create_button(ui, "生成字幕").clicked() {
                                        self.start_processing();
                                    }
                                }

                                AppStatus::Processing => {
                                    // 显示处理中的状态
                                    let info = self.progress_info.lock().unwrap();
                                    ui.label(RichText::new(&info.message).size(14.0));
                                }

                                AppStatus::Completed => {
                                    // 完成后，提示保存文件
                                    ui.label(RichText::new("字幕生成完成！").size(14.0));
                                    ui.add_space(13.0);

                                    if self.create_button(ui, "保存字幕文件").clicked() {
                                        self.save_subtitle_file();
                                    }
                                }

                                AppStatus::SaveSuccess => {
                                    // 保存成功后，显示成功信息和"下一个"按钮
                                    ui.label(RichText::new("保存成功").size(14.0));
                                    ui.add_space(13.0);

                                    if self.create_button(ui, "下一个").clicked() {
                                        self.reset_to_initial();
                                    }
                                }

                                AppStatus::Error(ref err) => {
                                    // 显示错误信息和重试按钮
                                    ui.colored_label(Color32::RED, RichText::new(err).size(14.0));
                                    ui.add_space(13.0);

                                    if self.create_button(ui, "重新开始").clicked() {
                                        self.reset_to_initial();
                                    }
                                }
                            }
                        },
                    );
                });
            });
    }
}

impl eframe::App for VideoSubtitleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置UI刷新率
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // 底部状态栏（保留为非常小的区域，仅用于显示状态信息）
        TopBottomPanel::bottom("bottom_panel")
            .min_height(20.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(0xE0, 0xE0, 0xE0))
                    .inner_margin(egui::style::Margin::symmetric(4.0, 0.0)),
            )
            .show(ctx, |ui| {
                self.render_bottom_panel(ui);
            });

        // 主面板
        CentralPanel::default().show(ctx, |ui| {
            self.render_main_panel(ui);
        });

        // 当字幕生成完成后，自动提示保存
        if self.progress_info.lock().unwrap().status == AppStatus::Completed {
            self.save_subtitle_file();
        }
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
