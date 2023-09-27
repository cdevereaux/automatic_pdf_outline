use lopdf::{Document, Error};

use crate::font_reader::PdfFontReader;

mod font_reader;
mod pdf_font;

// let outlines_id = doc.add_object(dictionary! {
//     "Type" => "Outlines",
// });

// let outline_entry_id = doc.add_object(dictionary!(
//     "Title" => Object::string_literal("MyTitle"),
//     "Parent" => outlines_id,
//     "Dest" => vec![page_id.into(), "XYZ".into(), Object::Null, Object::Null, Object::Null,]
// ));

fn main() -> Result<(), Error> {
    let doc = Document::load("test/abop.pdf")?;

    let fonts = doc.get_all_fonts().unwrap();

    for font in &fonts {
        println!("{}", font);
    }
    println!("{:?}", fonts.len());

    Ok(())
}
