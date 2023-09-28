use lopdf::{Document, Error};
use pdf_tools::{
    pdf_outline::print_outline, PdfFontReader, PdfOutlineGenerator, PdfOutlineInserter,
};

mod pdf_tools;

fn main() -> Result<(), Error> {
    let mut doc = Document::load("test/abop.pdf")?;

    let mut fonts = doc.get_all_fonts().unwrap();

    let mut outline_fonts = vec![];
    while let Some((font, _count)) = fonts.pop_first() {
        outline_fonts.push(font);
    }

    let heading_fonts = vec![
        outline_fonts[0].clone(),
        outline_fonts[5].clone(),
        outline_fonts[12].clone(),
    ];

    let outline = doc.generate_outline(&heading_fonts);

    print_outline(&outline);

    doc.insert_outline(&outline)?;

    doc.save("./test/test.pdf")?;

    Ok(())
}
