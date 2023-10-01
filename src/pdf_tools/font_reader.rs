use std::collections::{BTreeMap, BTreeSet};

use lopdf::{content::Operation, Document, Error, ObjectId};

use super::pdf_font::PdfFont;

pub trait PdfFontReader {
    fn get_all_fonts(&self) -> Result<BTreeMap<PdfFont, usize>, Error>;
}

pub const SET_TEXT_FONT: &str = "Tf";
pub const SET_TEXT_MATRIX: &str = "Tm";
pub const DISPLAY_TEXT_OPS: [&str; 4] = ["Tj", "'", "\"", "TJ"];

impl PdfFontReader for Document {
    fn get_all_fonts(&self) -> Result<BTreeMap<PdfFont, usize>, Error> {
        let mut base_fonts: BTreeSet<&str> = BTreeSet::new();
        let mut fonts = BTreeMap::new();
        for page_id in self.page_iter() {
            //Collect the names of every font on the current page
            for (_font_id, font) in self.get_page_fonts(page_id) {
                let base_font = font.get(b"BaseFont")?;
                let base_font_string = base_font.as_name_str()?;
                base_fonts.insert(base_font_string);
            }

            //record each font used on page
            let mut page_fonts = BTreeSet::new();
            let mut current_font = PdfFont::default();
            let contents = self.get_and_decode_page_content(page_id)?;
            for op in contents.operations {
                match op.operator.as_str() {
                    SET_TEXT_MATRIX | SET_TEXT_FONT => {
                        update_font_from_operation(self, &mut current_font, op, page_id)?
                    }
                    x if DISPLAY_TEXT_OPS.contains(&x) => {
                        page_fonts.insert(current_font.clone());
                    }
                    _ => (),
                }
            }

            //add page fonts to count
            for font in page_fonts {
                fonts
                    .entry(font.clone())
                    .and_modify(|entry| *entry += 1)
                    .or_insert(1);
            }
        }
        Ok(fonts)
    }
}

pub fn update_font_from_operation(
    doc: &Document,
    font: &mut PdfFont,
    op: Operation,
    page_id: ObjectId,
) -> Result<(), Error> {
    match op.operator.as_str() {
        SET_TEXT_MATRIX => match &op.operands[..] {
            [a, _b, _c, d, _e, _f] => {
                font.set_size((a.as_float()?, d.as_float()?));
            }
            _ => return Err(Error::Syntax(String::from("Invalid Tm operands"))),
        },
        SET_TEXT_FONT => match &op.operands[..] {
            [new_font, size] => {
                font.set_base_font(
                    doc.get_page_fonts(page_id)
                        .get(new_font.as_name()?)
                        .ok_or(Error::ObjectNotFound)?
                        .get(b"BaseFont")?
                        .as_name_str()?
                        .to_string(),
                );

                let size = size.as_float()?;
                font.set_size((size, size));
            }
            _ => return Err(Error::Syntax(String::from("Invalid Tf operands"))),
        },
        _ => panic!("update_font_from_operation may only be called with Tf or Tm operators"),
    }
    Ok(())
}
