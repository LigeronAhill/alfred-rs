CREATE TABLE IF NOT EXISTS users
(
    user_id BIGINT PRIMARY KEY,
    user_name VARCHAR NOT NULL,
    user_role VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
)
