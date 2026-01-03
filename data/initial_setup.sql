INSERT INTO
    roles (name)
VALUES
    ('Admin'),
    ('User')
ON CONFLICT DO NOTHING;

INSERT INTO
    users (name, email, password_hash, role_id)
SELECT
    'Eleazar Fig',
    'eleazar.fig@example.com',
    '$2b$12$L3ySTHvV8OcJ597/8wUw.OgEwgpZXL02f2.s7nC7hdLDrN5e.iwji',
    role_id
FROM
    roles
WHERE
    name LIKE 'Admin';

