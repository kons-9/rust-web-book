INSERT INTO 
    books (
        book_id,
        title,
        author,
        isbn,
        description,
        user_id,
        created_at,
        updated_at
    )
VALUES
    (
    '5b4c96ac-316a-4bee-8e69-cac5eb84ff4d',
    'The Rust Programming Language', 
    'Steve Klabnik and Carol Nichols', 
    '9781593278281', 
    'A comprehensive guide to the Rust programming language.', 
    '5b4c96ac-316a-4bee-8e69-cac5eb84ff4c',
    NOW(), 
    NOW()
    ) ON CONFLICT DO NOTHING;