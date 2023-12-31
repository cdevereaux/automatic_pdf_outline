use lopdf::{Document, ObjectId};

use super::{
    font_reader::{update_font_from_operation, DISPLAY_TEXT_OPS, SET_TEXT_FONT, SET_TEXT_MATRIX},
    pdf_font::PdfFont,
    pdf_outline::{PdfOutline, PdfOutlineEntry},
};

pub trait PdfOutlineGenerator {
    fn generate_outline(&self, fonts: &[Vec<PdfFont>]) -> PdfOutline;
}

impl PdfOutlineGenerator for Document {
    fn generate_outline(&self, heading_fonts: &[Vec<PdfFont>]) -> PdfOutline {
        let mut outline = PdfOutline::new();
        const MAX_DEPTH: usize = 3;
        for (current_depth, fonts) in heading_fonts.iter().enumerate() {
            if current_depth >= MAX_DEPTH {
                break;
            }
            'page_loop: for (page_number, page_id) in self.get_pages() {
                for font in fonts.iter() {
                    if let Some(title) = get_first_instance_on_page(self, page_id, font) {
                        let mut parent = &mut outline;
                        for _depth in 0..current_depth {
                            if let Some(entry) = parent
                                .iter_mut()
                                .take_while(|entry| entry.page_number <= page_number)
                                .last()
                            {
                                parent = &mut entry.children;
                            } else {
                                continue 'page_loop;
                            }
                        }

                        parent.push(PdfOutlineEntry::new(page_number, title));
                    }
                }
            }
        }
        outline
    }
}

fn get_first_instance_on_page(doc: &Document, page_id: ObjectId, font: &PdfFont) -> Option<String> {
    let mut first_instance = String::default();
    let mut current_font = PdfFont::default();

    let contents = doc.get_and_decode_page_content(page_id).ok()?;
    for op in contents.operations {
        match op.operator.as_str() {
            SET_TEXT_MATRIX | SET_TEXT_FONT => {
                if first_instance.is_empty() {
                    update_font_from_operation(doc, &mut current_font, op, page_id).ok()?
                } else {
                    return Some(first_instance);
                }
            }
            x if current_font == *font && DISPLAY_TEXT_OPS.contains(&x) => {
                if let Some(string_object) = match x {
                    "Tj" | "'" => op.operands.get(0),
                    "\"" => op.operands.get(2),
                    "TJ" => op.operands.get(0)?.as_array().ok()?.get(0),
                    _ => unreachable!(),
                } {
                    first_instance.push_str(&string_object.as_string().ok()?);
                }
            }
            _ => (),
        }
    }
    if first_instance.is_empty() {
        None
    } else {
        Some(first_instance)
    }
}
