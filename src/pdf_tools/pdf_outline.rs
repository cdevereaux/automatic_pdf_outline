pub struct PdfOutlineEntry {
    pub page_number: u32,
    pub title: String,
    pub children: Vec<PdfOutlineEntry>,
}

impl PdfOutlineEntry {
    pub fn new(page_number: u32, title: String) -> Self {
        Self {
            page_number,
            title,
            children: Vec::<PdfOutlineEntry>::default(),
        }
    }
}

pub type PdfOutline = Vec<PdfOutlineEntry>;

pub fn print_outline(outline: &PdfOutline) {
    recursive_print_outline(outline, 0);
}

fn recursive_print_outline(outline: &PdfOutline, depth: usize) {
    for entry in outline {
        for _ in 0..depth {
            print!(" ");
        }
        println!("{}  {}", entry.title, entry.page_number);
        recursive_print_outline(&entry.children, depth + 1);
    }
}
