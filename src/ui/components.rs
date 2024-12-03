use eframe::egui;

pub fn show_section(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .outer_margin(egui::Margin::ZERO)
        .inner_margin(egui::Margin::same(8.0))
        .rounding(egui::Rounding::same(4.0))
        .fill(ui.style().visuals.extreme_bg_color)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading(title);
                ui.separator();
                ui.add_space(4.0);
                content(ui);
            });
        });
}

pub fn show_text_list(ui: &mut egui::Ui, items: &[String], is_interactive: bool) {
    if let [] = items {
        draw_textarea(ui, "無資料", false);
    } else {
        items.iter().for_each(|item| draw_textarea(ui, item, is_interactive));
    }
}

fn draw_textarea(ui: &mut egui::Ui, text: &str, is_interactive: bool) {
    let mut text_copy = text.to_string();
    let text_edit = egui::TextEdit::multiline(&mut text_copy)
        .desired_width(ui.available_width())
        .frame(true)
        .interactive(is_interactive);

    egui::Frame::none()
        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
        .rounding(egui::Rounding::same(2.0))
        .show(ui, |ui| {
            ui.add(text_edit);
        });
}
