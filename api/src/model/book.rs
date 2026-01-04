use chrono::{DateTime, Utc};
use derive_new::new;
use garde::Validate;
use kernel::model::{
    book::{
        Book, BookListOptions, Checkout,
        event::{CreateBook, UpdateBook},
    },
    id::{BookId, CheckoutId, UserId},
    list::PaginatedList,
};
use serde::{Deserialize, Serialize};

use crate::model::user::{BookOwner, CheckoutUser};

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(length(min = 1))]
    pub author: String,
    #[garde(length(min = 1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

impl From<CreateBookRequest> for CreateBook {
    fn from(req: CreateBookRequest) -> Self {
        let CreateBookRequest {
            title,
            author,
            isbn,
            description,
        } = req;
        CreateBook {
            title,
            author,
            isbn,
            description,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookRequest {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(length(min = 1))]
    pub author: String,
    #[garde(length(min = 1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

#[derive(new)]
pub struct UpdateBookRequestWithId(BookId, UserId, UpdateBookRequest);

impl From<UpdateBookRequestWithId> for UpdateBook {
    fn from(value: UpdateBookRequestWithId) -> Self {
        let UpdateBookRequestWithId(book_id, requested_user, request) = value;
        let UpdateBookRequest {
            title,
            author,
            isbn,
            description,
        } = request;
        UpdateBook {
            book_id,
            title,
            author,
            isbn,
            description,
            requested_user,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct BookListQuery {
    #[garde(range(min = 0))]
    #[serde(default = "default_limit")]
    pub limit: i64,

    #[garde(range(min = 0))]
    #[serde(default)]
    pub offset: i64,
}

const DEFAULT_LIMIT: i64 = 20;
const fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

impl From<BookListQuery> for BookListOptions {
    fn from(value: BookListQuery) -> Self {
        let BookListQuery { limit, offset } = value;
        BookListOptions { limit, offset }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owner: BookOwner,
    pub checkout: Option<BookCheckoutResponse>,
}

impl From<Book> for BookResponse {
    fn from(book: Book) -> Self {
        let Book {
            id,
            title,
            author,
            isbn,
            description,
            owner,
            checkout,
        } = book;
        BookResponse {
            id,
            title,
            author,
            isbn,
            description,
            owner: owner.into(),
            checkout: checkout.map(BookCheckoutResponse::from),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedBookResponse {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub items: Vec<BookResponse>,
}

impl From<PaginatedList<Book>> for PaginatedBookResponse {
    fn from(paginated: PaginatedList<Book>) -> Self {
        let PaginatedList {
            total,
            limit,
            offset,
            items,
        } = paginated;
        PaginatedBookResponse {
            total,
            limit,
            offset,
            items: items.into_iter().map(BookResponse::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookCheckoutResponse {
    pub id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    pub checked_out_at: DateTime<Utc>,
}

impl From<Checkout> for BookCheckoutResponse {
    fn from(value: Checkout) -> Self {
        let Checkout {
            checkout_id,
            checked_out_by,
            checked_out_at,
        } = value;
        BookCheckoutResponse {
            id: checkout_id,
            checked_out_by: checked_out_by.into(),
            checked_out_at,
        }
    }
}
