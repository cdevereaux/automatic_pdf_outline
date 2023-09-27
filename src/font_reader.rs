use std::collections::BTreeSet;

use lopdf::{Document, Error};

use crate::pdf_font::PdfFont;

pub trait PdfFontReader {
    fn get_all_fonts(&self) -> Result<BTreeSet<PdfFont>, Error>;
}

const SET_TEXT_FONT: &str = "Tf";
const SET_TEXT_MATRIX: &str = "Tm";
const DISPLAY_TEXT_OPS: [&str; 4] = ["Tj", "'", "\"", "TJ"];

impl PdfFontReader for Document {
    fn get_all_fonts(&self) -> Result<BTreeSet<PdfFont>, Error> {
        let mut base_fonts: BTreeSet<&str> = BTreeSet::new();
        let mut fonts = BTreeSet::new();
        for page_id in self.page_iter() {
            //Collect the names of every font on the current page
            for (_font_id, font) in self.get_page_fonts(page_id) {
                let base_font = font.get(b"BaseFont")?;
                let base_font_string = base_font.as_name_str()?;
                base_fonts.insert(base_font_string);
            }

            let mut current_font = PdfFont::default();

            let contents = self.get_and_decode_page_content(page_id)?;
            for op in contents.operations {
                match op.operator.as_str() {
                    SET_TEXT_MATRIX => match &op.operands[..] {
                        [a, _b, _c, d, _e, _f] => {
                            current_font.set_size((a.as_float()?, d.as_float()?));
                        }
                        _ => return Err(Error::Syntax(String::from("Invalid Tm operands"))),
                    },
                    SET_TEXT_FONT => match &op.operands[..] {
                        [font, size] => {
                            current_font.set_base_font(
                                self.get_page_fonts(page_id)
                                    .get(font.as_name()?)
                                    .ok_or(Error::ObjectNotFound)?
                                    .get(b"BaseFont")?
                                    .as_name_str()?
                                    .to_string(),
                            );

                            let size = size.as_float()?;
                            current_font.set_size((size, size));
                        }
                        _ => return Err(Error::Syntax(String::from("Invalid Tf operands"))),
                    },
                    x if DISPLAY_TEXT_OPS.contains(&x) => {
                        fonts.insert(current_font.clone());
                    }
                    _ => (),
                }
            }
        }
        Ok(fonts)
    }
}
