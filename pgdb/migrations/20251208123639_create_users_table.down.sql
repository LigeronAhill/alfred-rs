-- Удаление представления
DROP VIEW IF EXISTS user_details;

-- Удаление триггеров
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP TRIGGER IF EXISTS update_user_infos_updated_at ON user_infos;

-- Удаление функции для обновления updated_at
DROP FUNCTION IF EXISTS update_updated_at_column;

-- Удаление таблиц (в правильном порядке из-за foreign key constraints)
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS user_infos;

-- Удаление ENUM типа (должно быть последним)
DROP TYPE IF EXISTS user_role CASCADE;
