use eframe::NativeOptions;
use egui::ViewportBuilder;
use video_subtitle::gui::VideoSubtitleApp;

fn main() -> eframe::Result<()> {
    // 环境日志初始化
    env_logger::init();

    // 应用选项
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_min_inner_size([540.0, 380.0])
            .with_title("视频字幕提取工具"),
        ..Default::default()
    };

    // 运行应用
    eframe::run_native(
        "视频字幕提取工具",
        options,
        Box::new(|cc| Box::new(VideoSubtitleApp::new(cc))),
    )
}
