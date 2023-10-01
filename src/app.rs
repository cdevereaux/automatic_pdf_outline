use egui::RichText;
use egui_extras::{Column, TableBuilder};
use lopdf::Document;

use crate::{
    pdf_tools::{
        pdf_font::PdfFont, pdf_outline::PdfOutline, PdfFontReader, PdfOutlineGenerator,
        PdfOutlineInserter,
    },
    save_file::save_file_from_rust,
};

#[derive(Debug, PartialEq)]
enum OutlineLevel {
    First,
    Second,
    Third,
    None,
}

#[derive(Debug)]
struct FontRow {
    font: PdfFont,
    count: usize,
    level: OutlineLevel,
}

#[derive(Debug, Default)]
pub struct App {
    file_name: String,
    fonts: Option<Vec<FontRow>>,
    heading_fonts: [Vec<PdfFont>; 3],
    outline: Option<PdfOutline>,
    doc: Option<Document>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Default::default()
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.doc.is_none() {
            self.check_for_new_pdf_file(ctx);
        }

        egui::TopBottomPanel::top("Header").show(ctx, |ui| {
            if self.doc.is_some() {
                let mut string = String::from("Current File: ");
                string.push_str(&self.file_name);
                ui.heading(string);
            } else {
                ui.heading("Drag and Drop a PDF File");
            }
        });

        egui::TopBottomPanel::bottom("Buttons").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                let enabled =
                    self.fonts.is_some() && self.heading_fonts.iter().any(|v| !v.is_empty());
                ui.add_enabled_ui(enabled, |ui| {
                    if ui
                        .button(RichText::new("Generate Outline").heading())
                        .clicked()
                    {
                        let fonts = self.heading_fonts.to_vec();
                        self.outline = Some(self.doc.as_ref().unwrap().generate_outline(&fonts));
                    }
                });
                let enabled = self.outline.is_some();
                ui.add_enabled_ui(enabled, |ui| {
                    if ui
                        .button(RichText::new("Save PDF with Outline").heading())
                        .clicked()
                    {
                        let mut new_doc = self.doc.as_ref().unwrap().clone();
                        //todo: check this
                        new_doc
                            .insert_outline(self.outline.as_ref().unwrap())
                            .unwrap();
                        let mut data = vec![];
                        //todo: check this
                        new_doc.save_to(&mut data).unwrap();
                        save_file_from_rust(data);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.vertical(|ui| {
                    self.font_table(ui);
                });
                ui.vertical(|ui| {
                    if let Some(outline) = &self.outline {
                        ui.heading("Outline Preview");

                        egui::ScrollArea::vertical()
                            .id_source("Outline Scroll Area")
                            .show(ui, |ui| {
                                Self::outline_preview(ui, outline, 0);
                            });
                    }
                });
            });
        });
    }
}

impl App {
    fn check_for_new_pdf_file(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            for file in &input.raw.dropped_files {
                if file.mime.ends_with("pdf") {
                    if let Some(bytes) = &file.bytes {
                        if let Ok(doc) = Document::load_mem(bytes) {
                            self.doc = Some(doc);
                            self.file_name = file.name.clone();
                        }
                    }
                }
            }
        });
    }

    fn font_table(&mut self, ui: &mut egui::Ui) {
        if self.fonts.is_none() {
            if let Some(doc) = &self.doc {
                let fonts = doc.get_all_fonts().unwrap();
                self.fonts = Some(
                    fonts
                        .iter()
                        .map(|(k, &v)| FontRow {
                            font: k.clone(),
                            count: v,
                            level: OutlineLevel::None,
                        })
                        .collect(),
                );
            }
        } else {
            ui.heading("Fonts");

            TableBuilder::new(ui)
                .resizable(true)
                .striped(true)
                .column(Column::initial(100.0))
                .column(Column::initial(125.0))
                .column(Column::auto().resizable(true))
                .column(Column::exact(200.0))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Name");
                    });
                    header.col(|ui| {
                        ui.label("Size");
                    });
                    header.col(|ui| {
                        ui.label("Page Count");
                    });
                    header.col(|ui| {
                        ui.label("Outline Level");
                    });
                })
                .body(|body| {
                    if let Some(fonts) = &mut self.fonts {
                        body.rows(20.0, fonts.len(), |index, mut row| {
                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(&fonts[index].font.base_font).truncate(true),
                                );
                            });
                            row.col(|ui| {
                                let size = fonts[index].font.size;
                                let size_str = if size.0 == size.1 {
                                    format!("{:?}", fonts[index].font.size.0)
                                } else {
                                    format!("{:?}", fonts[index].font.size)
                                };
                                ui.add(egui::Label::new(size_str).truncate(true));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", fonts[index].count));
                            });
                            row.col(|ui| {
                                let level = &mut fonts[index].level;
                                Self::outline_level_buttons(ui, level);
                            });
                        });
                    }
                });
            self.update_heading_fonts();
        }
    }

    fn update_heading_fonts(&mut self) {
        self.heading_fonts = [vec![], vec![], vec![]];
        if let Some(fonts) = &self.fonts {
            for font in fonts {
                let index = match font.level {
                    OutlineLevel::First => 0,
                    OutlineLevel::Second => 1,
                    OutlineLevel::Third => 2,
                    OutlineLevel::None => continue,
                };
                self.heading_fonts[index].push(font.font.clone());
            }
        }
    }

    fn outline_level_buttons(ui: &mut egui::Ui, level: &mut OutlineLevel) {
        ui.horizontal(|ui| {
            for variant in [
                OutlineLevel::First,
                OutlineLevel::Second,
                OutlineLevel::Third,
            ] {
                let selected = *level == variant;
                if ui
                    .add(egui::RadioButton::new(selected, format!("{:?}", variant)))
                    .clicked()
                {
                    *level = if selected {
                        OutlineLevel::None
                    } else {
                        variant
                    };
                }
            }
        });
    }

    fn outline_preview(ui: &mut egui::Ui, outline: &PdfOutline, mut id: usize) {
        for entry in outline {
            ui.push_id(id, |ui| {
                egui::CollapsingHeader::new(entry.title.clone())
                    .default_open(true)
                    .show(ui, |ui| {
                        id += 1;
                        Self::outline_preview(ui, &entry.children, id);
                    });
            });
            id += entry.children.len() + 1;
        }
    }
}
