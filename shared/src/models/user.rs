use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub user_id: i64,
    pub user_name: String,
    pub user_role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum UserRole {
    Admin,
    Employee,
    Guest,
}
impl From<UserRole> for i32 {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Admin => 1,
            UserRole::Employee => 2,
            UserRole::Guest => 0,
        }
    }
}

const ADMIN: &str = "Администратор";
const EMPLOYEE: &str = "Сотрудник";
const GUEST: &str = "Гость";

impl From<String> for UserRole {
    fn from(role: String) -> Self {
        match role.as_str() {
            ADMIN => UserRole::Admin,
            EMPLOYEE => UserRole::Employee,
            _ => UserRole::Guest,
        }
    }
}
impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "{ADMIN}"),
            UserRole::Employee => write!(f, "{EMPLOYEE}"),
            UserRole::Guest => write!(f, "{GUEST}"),
        }
    }
}
