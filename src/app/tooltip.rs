use crate::data::tooltip::TooltipData;
use crate::types::{Modification, ObjectType};
use crate::ui::fonts::setup_custom_fonts;
use bstr::BString;
use eframe::egui;
use std::path::PathBuf;

pub struct TooltipApp {
    data: TooltipData,
    status: String,
    search_text: String,
}

impl TooltipApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let data = TooltipData::new();
        let status = String::new();
        let search_text = String::new();
        Self {
            data,
            status,
            search_text,
        }
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.render_skill_filter(ui);
                self.render_skill_selector(ui);
                self.render_action_buttons(ui);
            });
            ui.add_space(1.0);
        });
    }

    fn render_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(&self.status));
            });
        });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    self.render_scroll_area(ui);
                });
        });
    }

    fn render_skill_filter(&mut self, ui: &mut egui::Ui) {
        let text_edit =
            egui::TextEdit::singleline(&mut self.search_text).hint_text("輸入技能ID後按Enter搜尋");

        if ui.add(text_edit).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if self.data.current_id == self.search_text {
                return;
            }

            match self
                .data
                .object_manager
                .is_exist(&BString::from(self.search_text.as_str()))
            {
                true => {
                    self.data.current_id = self.search_text.clone();
                    self.update_status(format!("已切換技能組至[{}]", self.search_text));
                }
                false => {
                    self.update_status("找不到技能ID");
                }
            }
        }
    }

    fn render_skill_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("skill_selector")
            .selected_text(&self.data.current_id)
            .show_ui(ui, |ui| {
                for id in self.data.object_manager.get_skill_ids() {
                    if ui
                        .selectable_value(
                            &mut self.data.current_id,
                            id.to_string(),
                            &id.to_string(),
                        )
                        .clicked()
                    {
                        self.update_status(format!("已切換技能組至[{id}]"))
                    }
                }
            });
    }

    fn render_action_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("開啟").clicked() {
            if let Some(path) = pick_war3_file() {
                match self.data.import(&path) {
                    Ok(_) => {
                        self.update_status("讀取檔案成功");
                    }
                    Err(e) => {
                        self.update_status(format!("讀取失敗：{}", e));
                    }
                }
            } else {
                self.update_status("沒有選擇檔案");
            }
        }

        if ui.button("匯出").clicked() {
            match self.data.export(false) {
                Ok(e) => self.update_status(format!("匯出成功，匯出路徑: {}", e)),
                Err(e) => self.update_status(format!("匯出失敗: {}", e)),
            }
        }

        if ui.button("儲存翻譯").clicked() {
            match self.data.export(true) {
                Ok(e) => self.update_status(format!("儲存成功，儲存路徑:{}", e)),
                Err(e) => self.update_status(format!("儲存失敗: {}", e)),
            }
        }

        if ui.button("新增/重置翻譯").clicked() {
            match self.data.add_localized(self.data.current_id.clone()) {
                Ok(_) => self.update_status("已新增/重置翻譯當前技能的翻譯內容"),
                Err(e) => self.update_status(format!("操作失敗：{}", e)),
            }
        }
    }

    fn render_scroll_area(&mut self, ui: &mut egui::Ui) {
        let manager = &mut self.data.object_manager;
        if manager.is_empty() {
            egui::Frame::none().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(32.0);
                    ui.label("無資料");
                    ui.add_space(32.0);
                });
            });
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("main_scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let (source_data, localized_data) =
                        manager.get_data_mut(&BString::from(self.data.current_id.as_str()));

                    if let Some(data) = source_data {
                        show_split_section(ui, self.data.current_id.as_str(), data, localized_data);
                    }
                });
            });
    }

    fn update_status(&mut self, str: impl Into<String>) {
        self.status = str.into();
    }
}

fn pick_war3_file() -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    dialog = dialog.add_filter("War3 Object Files", &ObjectType::all_extensions());
    dialog.pick_file()
}

fn show_split_section(
    ui: &mut egui::Ui,
    id: &str,
    source_data: &mut Vec<Modification>,
    localized_data: Option<&mut Vec<Modification>>,
) {
    ui.push_id("split_section", |ui| {
        let frame = egui::Frame::group(ui.style())
            .outer_margin(egui::Margin::ZERO)
            .inner_margin(egui::Margin::same(8.0))
            .rounding(egui::Rounding::same(4.0))
            .fill(ui.style().visuals.extreme_bg_color);

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading(id);
                ui.separator();
                ui.add_space(4.0);
                render_split_columns(ui, source_data, localized_data);
            });
        });
    });
}

fn render_split_columns(
    ui: &mut egui::Ui,
    source_data: &mut Vec<Modification>,
    localized_data: Option<&mut Vec<Modification>>,
) {
    let column_width = (ui.available_width() - 20.0) / 2.0;

    egui::Grid::new("split_content")
        .num_columns(2)
        .spacing([20.0, 0.0])
        .show(ui, |ui| {
            render_column(ui, column_width, Some(source_data), false);
            render_column(ui, column_width, localized_data, true);
            ui.end_row();
        });
}

fn render_column(
    ui: &mut egui::Ui,
    width: f32,
    item: Option<&mut Vec<Modification>>,
    is_editable: bool,
) {
    ui.vertical(|ui| {
        ui.set_width(width);

        if let Some(data) = item {
            let mut temp = "1234";
            let text_edit = egui::TextEdit::multiline(&mut temp)
                .desired_width(ui.available_width())
                .frame(true)
                .interactive(is_editable);

            egui::Frame::none()
                .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                .rounding(egui::Rounding::same(2.0))
                .show(ui, |ui| {
                    ui.add(text_edit);
                });
        }
    });
}

impl eframe::App for TooltipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_top_panel(ctx);
        self.render_bottom_panel(ctx);
        self.render_central_panel(ctx);
    }
}
