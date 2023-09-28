use lopdf::{dictionary, Document, Error, Object, ObjectId};

use super::pdf_outline::{PdfOutline, PdfOutlineEntry};

trait PdfOutlineEntryInserter {
    fn insert_outline_entries(
        &mut self,
        parent_id: ObjectId,
        children: &[PdfOutlineEntry],
    ) -> (ObjectId, ObjectId, i32);
}

impl PdfOutlineEntryInserter for Document {
    fn insert_outline_entries(
        &mut self,
        parent_id: ObjectId,
        children: &[PdfOutlineEntry],
    ) -> (ObjectId, ObjectId, i32) {
        let mut entry_ids = vec![];

        for entry in children {
            let entry_id = self.add_object(dictionary!(
                "Title" => Object::string_literal(entry.title.clone()),
                "Parent" => parent_id,
                "Dest" => vec![ (entry.page_number-1).into(), "XYZ".into(), Object::Null, Object::Null, Object::Null,],
            ));

            entry_ids.push(entry_id);

            if !entry.children.is_empty() {
                let (first, last, count) = self.insert_outline_entries(entry_id, &entry.children);
                let entry = self.get_dictionary_mut(entry_id).unwrap();

                entry.set("First", first);
                entry.set("Last", last);
                entry.set("Count", -count);
            }
        }

        for i in 0..entry_ids.len() {
            if i != 0 {
                self.get_dictionary_mut(entry_ids[i])
                    .unwrap()
                    .set("Prev", entry_ids[i - 1]);
            }
            if i != entry_ids.len() - 1 {
                self.get_dictionary_mut(entry_ids[i])
                    .unwrap()
                    .set("Next", entry_ids[i + 1]);
            }
        }

        (
            entry_ids[0],
            entry_ids[entry_ids.len() - 1],
            entry_ids.len() as i32,
        )
    }
}

pub trait PdfOutlineInserter {
    fn insert_outline(&mut self, outline: &PdfOutline) -> Result<(), Error>;
}

impl PdfOutlineInserter for Document {
    fn insert_outline(&mut self, outline: &PdfOutline) -> Result<(), Error> {
        let outlines_id = self.add_object(dictionary! {
            "Type" => "Outlines",
        });

        if !outline.is_empty() {
            let (first, last, _) = self.insert_outline_entries(outlines_id, outline);
            self.get_dictionary_mut(outlines_id)
                .unwrap()
                .set("First", first);
            self.get_dictionary_mut(outlines_id)
                .unwrap()
                .set("Last", last);
        }

        self.catalog_mut()?.set("Outlines", outlines_id);
        Ok(())
    }
}
