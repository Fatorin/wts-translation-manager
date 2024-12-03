use eframe::egui;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 根據作業系統選擇字體路徑
    let (chinese_font, japanese_font, korean_font) = match std::env::consts::OS {
        "windows" => (
            "C:\\Windows\\Fonts\\msyh.ttc",     // Windows 微軟雅黑
            "C:\\Windows\\Fonts\\msgothic.ttc", // Windows MS Gothic
            "C:\\Windows\\Fonts\\malgun.ttf",   // Windows Malgun Gothic
        ),
        "macos" => (
            "/System/Library/Fonts/PingFang.ttc",         // macOS 蘋方
            "/System/Library/Fonts/Hiragino Sans GB.ttc", // macOS Hiragino
            "/System/Library/Fonts/AppleSDGothicNeo.ttc", // macOS Apple SD Gothic Neo
        ),
        "linux" => (
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", // Linux Noto Sans CJK
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", // Linux Noto Sans CJK
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", // Linux Noto Sans CJK
        ),
        _ => {
            println!("不支援的作業系統，使用預設字體");
            return;
        }
    };

    // 載入字體
    if let Ok(chinese_data) = std::fs::read(chinese_font) {
        fonts.font_data.insert(
            "chinese_font".to_owned(),
            egui::FontData::from_owned(chinese_data),
        );
    }

    if let Ok(japanese_data) = std::fs::read(japanese_font) {
        fonts.font_data.insert(
            "japanese_font".to_owned(),
            egui::FontData::from_owned(japanese_data),
        );
    }

    if let Ok(korean_data) = std::fs::read(korean_font) {
        fonts.font_data.insert(
            "korean_font".to_owned(),
            egui::FontData::from_owned(korean_data),
        );
    }

    // 設定字體優先順序
    for family in &mut fonts.families.values_mut() {
        // 只加入成功載入的字體
        if fonts.font_data.contains_key("chinese_font") {
            family.insert(0, "chinese_font".to_owned());
        }
        if fonts.font_data.contains_key("japanese_font") {
            family.insert(1, "japanese_font".to_owned());
        }
        if fonts.font_data.contains_key("korean_font") {
            family.insert(2, "korean_font".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}