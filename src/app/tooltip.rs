use crate::data::tooltip::{SkillData, SkillManager, TooltipData};
use crate::ui::fonts::setup_custom_fonts;
use crate::utils::common::{RESEARCHTIP, RESEARCHUBERTIP, TIP, UBERTIP};
use crate::utils::export::{export_translated_content, save_translation};
use crate::utils::parser;
use eframe::egui;
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_SKILL: SkillData = SkillData::default();
}

const SOURCE_FILE_NAME: &str = "source.ini";
const TRANSLATE_FILE_NAME: &str = "translation.ini";

pub struct TooltipApp {
    data: TooltipData,
    status: String,
    search_text: String,
}

impl TooltipApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let data = parser::parse_tooltip_files(SOURCE_FILE_NAME, TRANSLATE_FILE_NAME);
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

            if self
                .data
                .skill_manager
                .skills
                .contains_key(&self.search_text)
            {
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
                for id in self.data.skill_manager.get_skill_ids() {
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
            match export_translated_content(&self.data, SOURCE_FILE_NAME) {
                Ok(_) => self.update_status("匯出成功"),
                Err(e) => self.update_status(format!("匯出失敗: {}", e)),
            }
        }

        if ui.button("存檔翻譯").clicked() {
            match save_translation(&self.data, TRANSLATE_FILE_NAME) {
                Ok(_) => self.update_status("存檔成功"),
                Err(e) => self.update_status(format!("存檔失敗: {}", e)),
            }
        }

        if ui.button("新增/重置翻譯").clicked() {
            if let Some(skill) = self
                .data
                .skill_manager
                .skills
                .get(&self.data.current_id)
                .cloned()
            {
                self.data
                    .skill_manager
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
                ui.vertical(|ui| {
                    let manager = &mut self.data.skill_manager;
                    render_skill_sections(ui, manager, &self.data.current_id);
                });
            });
    }

    fn update_status(&mut self, str: impl Into<String>) {
        self.status = str.into();
    }
}

fn render_skill_sections(ui: &mut egui::Ui, manager: &mut SkillManager, id: &str) {
    let (source_data, localized_data) = manager.get_data_mut(id);

    let mut temp_skill = SkillData::default();
    let localized = localized_data.unwrap_or(&mut temp_skill);

    if let Some(data) = source_data {
        ui.spacing_mut().item_spacing.y = 16.0;
        show_split_section(
            ui,
            RESEARCHTIP,
            "researchtip_section",
            &mut data.researchtip,
            &mut localized.researchtip,
        );
        show_split_section(
            ui,
            RESEARCHUBERTIP,
            "researchubertip_section",
            &mut data.researchubertip,
            &mut localized.researchubertip,
        );
        show_split_section(ui, TIP, "tip_section", &mut data.tip, &mut localized.tip);
        show_split_section(
            ui,
            UBERTIP,
            "ubertip_section",
            &mut data.ubertip,
            &mut localized.ubertip,
        );
    } else {
        ui.add_space(32.0);
        ui.label("無法讀取資料");
        ui.add_space(32.0);
    }
}

fn show_split_section(
    ui: &mut egui::Ui,
    title: &str,
    section_id: &str,
    source_data: &mut Vec<String>,
    localized_data: &mut Vec<String>,
) {
    ui.push_id(section_id, |ui| {
        let frame = egui::Frame::group(ui.style())
            .outer_margin(egui::Margin::ZERO)
            .inner_margin(egui::Margin::same(8.0))
            .rounding(egui::Rounding::same(4.0))
            .fill(ui.style().visuals.extreme_bg_color);

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading(title);
                ui.separator();
                ui.add_space(4.0);
                render_split_columns(ui, source_data, localized_data);
            });
        });
    });
}

fn render_split_columns(
    ui: &mut egui::Ui,
    source_data: &mut Vec<String>,
    localized_data: &mut Vec<String>,
) {
    let column_width = (ui.available_width() - 20.0) / 2.0;

    egui::Grid::new("split_content")
        .num_columns(2)
        .spacing([20.0, 0.0])
        .show(ui, |ui| {
            render_column(ui, "原文：", column_width, source_data, false);
            render_column(ui, "翻譯：", column_width, localized_data, true);
            ui.end_row();
        });
}

fn render_column(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    items: &mut Vec<String>,
    is_editable: bool,
) {
    ui.vertical(|ui| {
        ui.set_width(width);
        ui.label(label);
        ui.add_space(4.0);

        for i in 0..items.len() {
            let mut item = items[i].clone();

            let text_edit = egui::TextEdit::multiline(&mut item)
                .desired_width(ui.available_width())
                .frame(true)
                .interactive(is_editable);

            let response = egui::Frame::none()
                .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                .rounding(egui::Rounding::same(2.0))
                .show(ui, |ui| ui.add(text_edit));

            if response.inner.changed() {
                items[i] = item;
            }
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
