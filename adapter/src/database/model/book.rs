use kernel::model::book::Book;
use uuid::Uuid;

pub struct BookRow {
    pub book_id: Uuid,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

impl From<BookRow> for Book {
    fn from(row: BookRow) -> Self {
        Self {
            id: row.book_id,
            title: row.title,
            author: row.author,
            isbn: row.isbn,
            description: row.description,
        }
    }
}
