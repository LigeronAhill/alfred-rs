-- Создание ENUM типа для ролей пользователей
CREATE TYPE user_role AS ENUM ('Owner', 'Admin', 'Employee', 'Guest');

-- Создание таблицы user_infos для хранения дополнительной информации пользователей
CREATE TABLE user_infos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(255),
    middle_name VARCHAR(255),
    last_name VARCHAR(255),
    username VARCHAR(255) UNIQUE,
    userpic_url TEXT,
    bio TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Создание таблицы users (основная информация)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'Guest',
    user_info_id UUID REFERENCES user_infos(id) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Создание индексов для улучшения производительности
-- Для таблицы users
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_user_info_id ON users(user_info_id);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Для таблицы user_infos
CREATE INDEX idx_user_infos_username ON user_infos(username);
CREATE INDEX idx_user_infos_full_name ON user_infos(first_name, last_name);
CREATE INDEX idx_user_infos_created_at ON user_infos(created_at);

-- Составной индекс для поиска по email и роли
CREATE INDEX idx_users_email_role ON users(email, role);

-- Триггерная функция для автоматического обновления updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Создание триггеров для обеих таблиц
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_infos_updated_at
    BEFORE UPDATE ON user_infos
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Создание представления для удобного доступа ко всей информации пользователя
CREATE OR REPLACE VIEW user_details AS
SELECT
    u.id,
    u.email,
    u.password_hash,
    u.role,
    u.created_at as user_created_at,
    u.updated_at as user_updated_at,
    ui.first_name,
    ui.middle_name,
    ui.last_name,
    ui.username,
    ui.userpic_url,
    ui.bio,
    ui.created_at as info_created_at,
    ui.updated_at as info_updated_at
FROM users u
LEFT JOIN user_infos ui ON u.user_info_id = ui.id;

-- Добавление комментариев к таблицам и колонкам
COMMENT ON TYPE user_role IS 'Тип роли пользователя системы';
COMMENT ON TABLE users IS 'Основная таблица пользователей системы';
COMMENT ON COLUMN users.id IS 'Уникальный идентификатор пользователя';
COMMENT ON COLUMN users.email IS 'Электронная почта пользователя (уникальная)';
COMMENT ON COLUMN users.password_hash IS 'Хэш пароля пользователя';
COMMENT ON COLUMN users.role IS 'Роль пользователя (Owner, Admin, Employee, Guest)';
COMMENT ON COLUMN users.user_info_id IS 'Ссылка на дополнительную информацию пользователя';
COMMENT ON COLUMN users.created_at IS 'Дата и время создания учетной записи';
COMMENT ON COLUMN users.updated_at IS 'Дата и время последнего обновления учетной записи';

COMMENT ON TABLE user_infos IS 'Таблица с дополнительной информацией о пользователях';
COMMENT ON COLUMN user_infos.id IS 'Уникальный идентификатор записи информации';
COMMENT ON COLUMN user_infos.first_name IS 'Имя пользователя';
COMMENT ON COLUMN user_infos.middle_name IS 'Отчество пользователя';
COMMENT ON COLUMN user_infos.last_name IS 'Фамилия пользователя';
COMMENT ON COLUMN user_infos.username IS 'Уникальное имя пользователя';
COMMENT ON COLUMN user_infos.userpic_url IS 'URL аватарки пользователя';
COMMENT ON COLUMN user_infos.bio IS 'Биография пользователя';
COMMENT ON COLUMN user_infos.created_at IS 'Дата и время создания информации';
COMMENT ON COLUMN user_infos.updated_at IS 'Дата и время последнего обновления информации';
