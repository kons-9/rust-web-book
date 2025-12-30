use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        book::{Book, event::CreateBook},
        id::BookId,
    },
    repository::book::BookRepository,
};
use shared::error::{AppError, AppResult};

use crate::database::{ConnectionPool, model::book::BookRow};

#[derive(new)]
pub struct BookRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {
    async fn create(&self, event: CreateBook) -> AppResult<()> {
        sqlx::query!(
            r#"
                INSERT INTO books (title, author, isbn, description)
                VALUES ($1, $2, $3, $4)
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }
    async fn find_all(&self) -> AppResult<Vec<Book>> {
        let rows: Vec<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT book_id, title, author, isbn, description
                FROM books
                ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>> {
        let row: Option<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT book_id, title, author, isbn, description
                FROM books
                WHERE book_id = $1
            "#,
            book_id as _
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(row.map(Into::into))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_register_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool));

        let book = CreateBook {
            title: "The Rust Programming Language".to_string(),
            author: "Steve Klabnik and Carol Nichols".to_string(),
            isbn: "9781593278281".to_string(),
            description: "A comprehensive guide to Rust programming.".to_string(),
        };

        repo.create(book).await?;

        let res = repo.find_all().await?;
        assert_eq!(res.len(), 1);

        let book_id = res[0].id;
        let fetched_book = repo.find_by_id(book_id).await?;
        assert!(fetched_book.is_some());

        let Book {
            id,
            title,
            author,
            isbn,
            description,
        } = fetched_book.unwrap();
        assert_eq!(id, book_id);
        assert_eq!(title, "The Rust Programming Language");
        assert_eq!(author, "Steve Klabnik and Carol Nichols");
        assert_eq!(isbn, "9781593278281");
        assert_eq!(description, "A comprehensive guide to Rust programming.");

        Ok(())
    }
}
