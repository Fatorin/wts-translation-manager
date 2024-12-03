use crate::data::tooltip::{SkillData, TooltipData};
use crate::ui::fonts::setup_custom_fonts;
use crate::utils::export::{export_translated_content, save_translation};
use crate::utils::parser;
use eframe::egui;

pub struct TooltipApp {
    data: TooltipData,
    status: String,
    search_text: String,
}

impl TooltipApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let data = parser::parse_tooltip_files("sample.ini", "translation.ini");
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
            ui.add_space(1.0);
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
            let search_text = self.search_text.as_str();

            if self.data.current_id == search_text {
                return;
            }

            if self.data.skills.contains_key(&self.search_text) {
                self.data.current_id = search_text.to_string();
                self.update_status(format!("已切換技能組至[{}]", self.search_text));
                //self.search_text.clear();
            } else {
                self.update_status("找不到技能ID");
            }
        }
    }

    fn render_skill_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("選擇技能")
            .selected_text(&self.data.current_id)
            .show_ui(ui, |ui| {
                for id in self.data.get_skill_ids() {
                    if ui
                        .selectable_value(&mut self.data.current_id, id.clone(), &id)
                        .clicked()
                    {
                        self.update_status(format!("已切換技能組至[{id}]"))
                    }
                }
            });
    }

    fn render_action_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("資料匯出").clicked() {
            match export_translated_content(&self.data, "sample.ini") {
                Ok(_) => self.update_status("匯出成功"),
                Err(e) => self.update_status(format!("匯出失敗: {}", e)),
            }
        }

        if ui.button("存檔翻譯").clicked() {
            match save_translation(&self.data, "translation.ini") {
                Ok(_) => self.update_status("存檔成功"),
                Err(e) => self.update_status(format!("存檔失敗: {}", e)),
            }
        }

        if ui.button("新增/重置翻譯").clicked() {
            if let Some(skill) = self.data.skills.get(&self.data.current_id).cloned() {
                self.data
                    .translation_skills
                    .insert(self.data.current_id.clone(), skill);
                self.update_status("已新增/重置翻譯當前技能的翻譯內容");
            }
        }
    }

    fn render_scroll_area(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("main_scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                self.render_skill_sections(ui);
            });
    }

    fn render_skill_sections(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 16.0;

            let skill = self.data.get_current_skill();
            let translation_skill = self.data.try_get_current_translation_skill();

            self.render_all_sections(ui, &skill, &translation_skill);
        });
    }

    fn render_all_sections(
        &self,
        ui: &mut egui::Ui,
        skill: &SkillData,
        translation_skill: &SkillData,
    ) {
        let sections = [
            (
                "Researchtip",
                "researchtip_section",
                &skill.researchtip,
                &translation_skill.researchtip,
            ),
            (
                "Researchubertip",
                "researchubertip_section",
                &skill.researchubertip,
                &translation_skill.researchubertip,
            ),
            ("Tip", "tip_section", &skill.tip, &translation_skill.tip),
            (
                "Ubertip",
                "ubertip_section",
                &skill.ubertip,
                &translation_skill.ubertip,
            ),
        ];

        for (title, section_id, original, translation) in sections {
            self.show_split_section(
                ui,
                title,
                section_id,
                |ui| self.data.show_text_list(ui, original, false),
                |ui| self.data.show_text_list(ui, translation, true),
            );
        }
    }

    fn show_split_section(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        section_id: &str,
        content: impl FnOnce(&mut egui::Ui),
        translation: impl FnOnce(&mut egui::Ui),
    ) {
        ui.push_id(section_id, |ui| {
            self.data.show_section(ui, title, |ui| {
                let total_width = ui.available_width();
                let column_width = (total_width - 20.0) / 2.0;

                egui::Grid::new("split_content")
                    .num_columns(2)
                    .spacing([20.0, 0.0])
                    .show(ui, |ui| {
                        // Left column (original)
                        ui.vertical(|ui| {
                            ui.set_width(column_width);
                            ui.label("原文：");
                            ui.add_space(4.0);
                            content(ui);
                        });

                        ui.vertical(|ui| {
                            ui.set_width(column_width);
                            ui.label("翻譯：");
                            ui.add_space(4.0);
                            translation(ui);
                        });
                        ui.end_row();
                    });
            });
        });
    }

    fn update_status(&mut self, str: impl Into<String>) {
        self.status = str.into();
    }
}

impl eframe::App for TooltipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_top_panel(ctx);
        self.render_bottom_panel(ctx);
        self.render_central_panel(ctx);
    }
}
