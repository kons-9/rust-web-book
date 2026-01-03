use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        book::{
            Book, BookListOptions,
            event::{CreateBook, DeleteBook, UpdateBook},
        },
        id::{BookId, UserId},
        list::PaginatedList,
    },
    repository::book::BookRepository,
};
use shared::error::{AppError, AppResult};

use crate::database::{
    ConnectionPool,
    model::book::{BookRow, PaginatedBookRow},
};

#[derive(new)]
pub struct BookRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {
    async fn create(&self, event: CreateBook, user_id: UserId) -> AppResult<()> {
        sqlx::query!(
            r#"
                INSERT INTO books (title, author, isbn, description, user_id)
                VALUES ($1, $2, $3, $4, $5)
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            user_id as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }
    async fn find_all(&self, options: BookListOptions) -> AppResult<PaginatedList<Book>> {
        let BookListOptions { limit, offset } = options;
        let rows: Vec<PaginatedBookRow> = sqlx::query_as!(
            PaginatedBookRow,
            r#"
                SELECT 
                    COUNT(*) OVER() AS "total!",
                        b.book_id AS id
                FROM books AS b
                ORDER BY b.created_at DESC
                LIMIT $1 OFFSET $2
            "#,
            limit as _,
            offset as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let total = rows.first().map(|r| r.total).unwrap_or_default();
        let book_ids = rows
            .first()
            .into_iter()
            .map(|r| r.id)
            .collect::<Vec<BookId>>();

        let rows: Vec<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT 
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description,
                    u.user_id AS owned_by,
                    u.name AS owner_name
                FROM books AS b
                INNER JOIN users AS u USING(user_id)
                WHERE b.book_id IN (SELECT * FROM UNNEST($1::uuid[]))
                ORDER BY b.created_at DESC
            "#,
            &book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let items = rows.into_iter().map(Book::from).collect();

        Ok(PaginatedList {
            total,
            limit,
            offset,
            items,
        })
    }
    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>> {
        let row: Option<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT 
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description,
                    u.user_id AS owned_by,
                    u.name AS owner_name
                FROM books AS b
                INNER JOIN users AS u USING(user_id)
                WHERE b.book_id = $1
            "#,
            book_id as _
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(row.map(Into::into))
    }
    async fn update(&self, event: UpdateBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE books
                SET
                    title = $1,
                    author = $2,
                    isbn = $3,
                    description = $4,
                    updated_at = CURRENT_TIMESTAMP(3)
                WHERE book_id = $5 AND user_id = $6
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            event.book_id as _,
            event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified book not found".into()));
        }
        Ok(())
    }
    async fn delete(&self, _event: DeleteBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM books
                WHERE book_id = $1 AND user_id = $2
            "#,
            _event.book_id as _,
            _event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified book not found".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use kernel::{model::user::event::CreateUser, repository::user::UserRepository};

    use crate::repository::user::UserRepositoryImpl;

    use super::*;

    #[sqlx::test]
    async fn test_register_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        sqlx::query!(r#"INSERT INTO roles(name) VALUES ('Admin'), ('User');"#)
            .execute(&pool)
            .await?;
        let user_repo = UserRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool));

        let user = user_repo
            .create(CreateUser {
                name: "Test User".into(),
                email: "test@example.com".into(),
                password: "test_passwod".into(),
            })
            .await?;

        let book = CreateBook {
            title: "The Rust Programming Language".to_string(),
            author: "Steve Klabnik and Carol Nichols".to_string(),
            isbn: "9781593278281".to_string(),
            description: "A comprehensive guide to Rust programming.".to_string(),
        };
        repo.create(book, user.id).await?;

        let options = BookListOptions {
            limit: 20,
            offset: 0,
        };

        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);

        let book_id = res.items[0].id;
        let fetched_book = repo.find_by_id(book_id).await?;
        assert!(fetched_book.is_some());

        let Book {
            id,
            title,
            author,
            isbn,
            description,
            owner,
        } = fetched_book.unwrap();
        assert_eq!(id, book_id);
        assert_eq!(title, "The Rust Programming Language");
        assert_eq!(author, "Steve Klabnik and Carol Nichols");
        assert_eq!(isbn, "9781593278281");
        assert_eq!(description, "A comprehensive guide to Rust programming.");
        assert_eq!(owner.name, "Test User");

        Ok(())
    }
}
