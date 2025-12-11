mod pg_users_repository;
use crate::{
    AppResult,
    models::{SignupData, User},
};
use async_trait::async_trait;

#[async_trait]
pub trait UsersRepository {
    async fn create(&self, signup_data: SignupData) -> AppResult<User>;
    async fn get(&self, id: uuid::Uuid) -> AppResult<User>;
    async fn list(&self, page: u32, per_page: u32) -> AppResult<Vec<User>>;
    async fn total(&self) -> AppResult<u32>;
    async fn find_by_email(&self, email: &str) -> AppResult<User>;
    async fn update(&self, id: uuid::Uuid, user: User) -> AppResult<User>;
    async fn delete(&self, id: uuid::Uuid) -> AppResult<User>;
    async fn verify_user(&self, signup_data: SignupData) -> AppResult<bool>;
}
