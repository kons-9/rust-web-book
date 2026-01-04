use api::model::book::PaginatedBookResponse;
use axum::{body::Body, http::Request};
use kernel::model::user::BookOwner;
use kernel::{
    model::{book::Book, id::BookId, list::PaginatedList},
    repository::book::MockBookRepository,
};
use rstest::rstest;
use std::sync::Arc;
use tower::util::ServiceExt;

use crate::helper::fixture;
use crate::helper::v1;

#[rstest]
#[case("/books", 20, 0)]
#[case("/books?limit=50", 50, 0)]
#[case("/books?limit=50&offset=20", 50, 20)]
#[case("/books?offset=10", 20, 10)]
#[tokio::test]
async fn show_book_list_with_query_200(
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
    #[case] expected_limit: i64,
    #[case] expected_offset: i64,
) -> anyhow::Result<()> {
    use crate::{
        deserialize_json,
        helper::{TestRequestExt, make_router},
    };

    let book_id = BookId::new();

    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_find_all().returning(move |opt| {
            let items = vec![Book {
                id: book_id,
                title: "The Rust Programming Language".to_string(),
                isbn: "9781593278281".to_string(),
                author: "Steve Klabnik and Carol Nichols".to_string(),
                description: "A comprehensive guide to Rust programming.".to_string(),
                owner: BookOwner {
                    id: kernel::model::id::UserId::new(),
                    name: "Alice".to_string(),
                },
                checkout: None,
            }];
            Ok(PaginatedList {
                total: 1,
                limit: opt.limit,
                offset: opt.offset,
                items,
            })
        });
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1(path)).bearer().body(Body::empty())?;
    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, PaginatedBookResponse);
    assert_eq!(result.limit, expected_limit);
    assert_eq!(result.offset, expected_offset);

    Ok(())
}
