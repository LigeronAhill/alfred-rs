WITH new_user_info AS (
            INSERT INTO user_infos (first_name, middle_name, last_name, username, userpic_url, bio)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
        ),
        new_user AS (
            INSERT INTO users (email, password_hash, role, user_info_id)
            VALUES ($7, $8, $9::user_role, (SELECT id FROM new_user_info))
            RETURNING *
        )
        SELECT
            nu.id,
            nu.email,
            nu.password_hash,
            nu.role as "role: UserRole",
            nu.user_info_id,
            nu.created_at,
            nu.updated_at
        FROM new_user nu
