pub mod users {
    tonic::include_proto!("proto.users.v1");
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("proto_descriptor");
}
// pub use users::*;
use crate::users::{GetUserResponse, User};
use shared::models::UserRole;

impl From<shared::models::User> for GetUserResponse {
    fn from(value: shared::models::User) -> Self {
        Self {
            user_id: value.user_id,
            user_name: value.user_name.clone(),
            user_role: value.user_role.to_string(),
            created_at: value.created_at.to_string(),
            updated_at: value.updated_at.to_string(),
        }
    }
}
impl From<shared::models::User> for User {
    fn from(value: shared::models::User) -> Self {
        Self {
            user_id: value.user_id,
            user_name: value.user_name.clone(),
            user_role: value.user_role.to_string(),
            created_at: value.created_at.to_string(),
            updated_at: value.updated_at.to_string(),
        }
    }
}
impl From<User> for shared::models::User {
    fn from(value: User) -> Self {
        let created_at = value.created_at.parse().unwrap_or_default();
        let updated_at = value.updated_at.parse().unwrap_or_default();
        Self {
            user_id: value.user_id,
            user_name: value.user_name.clone(),
            user_role: UserRole::from(value.user_role),
            created_at,
            updated_at,
        }
    }
}
impl From<GetUserResponse> for shared::models::User {
    fn from(value: GetUserResponse) -> Self {
        let created_at = value.created_at.parse().unwrap_or_default();
        let updated_at = value.updated_at.parse().unwrap_or_default();
        Self {
            user_id: value.user_id,
            user_name: value.user_name.clone(),
            user_role: UserRole::from(value.user_role),
            created_at,
            updated_at,
        }
    }
}
