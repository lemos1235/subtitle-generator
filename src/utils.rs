use std::path::Path;

/// 获取资源文件路径，兼容开发环境和打包后的环境
///
/// # 参数
///
/// * `resource_path` - 资源文件的相对路径，如 "assets/image.png"
///
/// # 返回值
///
/// 返回可访问的资源文件完整路径，如果找不到资源文件，则返回原始路径
///
/// # 示例
///
/// ```
/// use subtitle_generator::utils::get_resource_path;
///
/// let font_path = get_resource_path("assets/font.ttf");
/// std::fs::read(&font_path).expect("无法读取字体文件");
/// ```
pub fn get_resource_path(resource_path: &str) -> String {
    // 首先尝试查找开发环境中的路径
    if Path::new(resource_path).exists() {
        return resource_path.to_string();
    }

    // 然后尝试在可执行文件所在目录的Resources目录下查找
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // macOS应用包中的Resources目录
            let app_resources_path = exe_dir.join("../Resources").join(resource_path);

            if app_resources_path.exists() {
                return app_resources_path.to_string_lossy().to_string();
            }

            // 尝试相对于可执行文件的路径
            let relative_path = exe_dir.join(resource_path);
            if relative_path.exists() {
                return relative_path.to_string_lossy().to_string();
            }
        }
    }

    // 如果都找不到，返回原始路径
    resource_path.to_string()
}

/// 读取资源文件内容
///
/// # 参数
///
/// * `resource_path` - 资源文件的相对路径
///
/// # 返回值
///
/// 返回资源文件的内容，如果读取失败则返回错误
pub fn read_resource_file(resource_path: &str) -> Result<Vec<u8>, std::io::Error> {
    let path = get_resource_path(resource_path);
    std::fs::read(path)
}

/// 获取ffmpeg可执行文件的路径
///
/// 首先尝试在应用包内的可执行文件目录查找，
/// 然后尝试在Resources/bin目录查找，
/// 最后回退到系统路径中的ffmpeg
///
/// # 返回值
///
/// 返回ffmpeg可执行文件的路径
pub fn get_ffmpeg_path() -> String {
    // 尝试在应用包内查找ffmpeg可执行文件
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // 1. 尝试在同一目录下查找
            let ffmpeg_path = exe_dir.join("ffmpeg");
            if ffmpeg_path.exists() {
                return ffmpeg_path.to_string_lossy().to_string();
            }

            // 2. 尝试在Resources/bin目录下查找
            let resources_bin_path = exe_dir.join("../Resources/assets/bin/ffmpeg");
            if resources_bin_path.exists() {
                return resources_bin_path.to_string_lossy().to_string();
            }
        }
    }

    // 3. 回退到系统路径
    "ffmpeg".to_string()
}
