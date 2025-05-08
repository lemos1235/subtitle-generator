use eframe::{NativeOptions, Theme};
use egui::{IconData, ViewportBuilder};
use subtitle_generator::gui::VideoSubtitleApp;
use subtitle_generator::utils::get_resource_path;

fn main() -> eframe::Result<()> {
    // 环境日志初始化
    env_logger::init();

    // 加载应用图标
    let icon_path = get_resource_path("assets/appicon.png");
    let logo = match image::open(&icon_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("无法加载应用图标: {}: {}", icon_path, e);
            // 如果无法加载图标，继续运行程序但不设置图标
            image::DynamicImage::new_rgba8(32, 32)
        }
    };

    // 提前获取logo的尺寸
    let width = logo.width();
    let height = logo.height();
    let rgba = logo.into_rgba8().into_raw();

    // 应用选项
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([350.0, 224.0])
            .with_min_inner_size([350.0, 224.0])
            .with_title("字幕生成器器")
            .with_icon(IconData {
                rgba,
                width,
                height,
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
