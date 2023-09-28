use std::collections::BTreeSet;

use lopdf::{Document, ObjectId};

use crate::{
    font_reader::{update_font_from_operation, DISPLAY_TEXT_OPS, SET_TEXT_FONT, SET_TEXT_MATRIX},
    pdf_font::PdfFont,
};

pub struct PdfOutlineEntry {
    page_number: u32,
    title: String,
    children: Vec<PdfOutlineEntry>,
}

impl PdfOutlineEntry {
    fn new(page_number: u32, title: String) -> Self {
        Self {
            page_number,
            title,
            children: Vec::<PdfOutlineEntry>::default(),
        }
    }
}

pub type PdfOutline = Vec<PdfOutlineEntry>;

pub fn print_outline(outline: PdfOutline) {
    recursive_print_outline(outline, 0);
}

fn recursive_print_outline(outline: PdfOutline, depth: usize) {
    for entry in outline {
        for _ in 0..depth {
            print!(" ");
        }
        println!("{}  {}", entry.title, entry.page_number);
        recursive_print_outline(entry.children, depth + 1);
    }
}

pub fn generate_outline(doc: &Document, fonts: &BTreeSet<PdfFont>) -> PdfOutline {
    let mut outline = PdfOutline::new();
    const MAX_DEPTH: usize = 3;
    for (current_depth, font) in fonts.iter().enumerate() {
        if current_depth >= MAX_DEPTH {
            break;
        }
        'page_loop: for (page_number, page_id) in doc.get_pages() {
            if let Some(title) = get_first_instance_on_page(&doc, page_id, font) {
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
    outline
}

fn get_first_instance_on_page(doc: &Document, page_id: ObjectId, font: &PdfFont) -> Option<String> {
    let mut current_font = PdfFont::default();

    let contents = doc.get_and_decode_page_content(page_id).ok()?;
    for op in contents.operations {
        match op.operator.as_str() {
            SET_TEXT_MATRIX | SET_TEXT_FONT => {
                update_font_from_operation(&doc, &mut current_font, op, page_id).ok()?
            }
            x if current_font == *font && DISPLAY_TEXT_OPS.contains(&x) => {
                if let Some(string_object) = match x {
                    "Tj" | "'" => op.operands.get(0),
                    "\"" => op.operands.get(2),
                    "TJ" => op.operands.get(0)?.as_array().ok()?.get(0),
                    _ => unreachable!(),
                } {
                    return string_object
                        .as_string()
                        .and_then(|s| Ok(s.to_string()))
                        .ok();
                }
            }
            _ => (),
        }
    }
    None
}
