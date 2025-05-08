use eframe::{NativeOptions, Theme};
use egui::{IconData, ViewportBuilder};
use subtitle_generator::gui::VideoSubtitleApp;

fn main() -> eframe::Result<()> {
    // 环境日志初始化
    env_logger::init();
    let logo = image::open("assets/appicon.png").unwrap();
    // 应用选项
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([350.0, 224.0])
            .with_min_inner_size([350.0, 224.0])
            .with_title("字幕生成器器")
            .with_icon(IconData{
                rgba: logo.into_rgba8().into_raw(),
                width: 1024,
                height: 1024,
            })
            .with_decorations(true)
            .with_transparent(false),
        default_theme: Theme::Light,
        follow_system_theme: false,
        ..Default::default()
    };

    // 运行应用
    eframe::run_native(
        "字幕生成器器",
        options,
        Box::new(|cc| Box::new(VideoSubtitleApp::new(cc))),
    )
}
