use anyhow::Result;
use proto::users::{
    GetUserRequest, ListAllUsersRequest, RegisterUserRequest,
    users_service_client::UsersServiceClient,
};
use tonic::Request;

const ADDRESS: &'static str = "http://[::1]:50051";
const AUTH: &'static str = "authorization";

#[derive(Clone)]
pub struct UsersClient {
    token: tonic::metadata::MetadataValue<tonic::metadata::Ascii>,
}
impl UsersClient {
    pub fn new(token: &str) -> Result<Self> {
        Ok(Self {
            token: token.parse()?,
        })
    }
    pub async fn register_new_user(
        &self,
        user_id: u64,
        user_name: String,
    ) -> Result<shared::models::UserRole> {
        let channel = tonic::transport::Channel::from_static(ADDRESS)
            .connect()
            .await?;
        let mut users_client =
            UsersServiceClient::with_interceptor(channel, move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert(AUTH, self.token.clone());
                Ok(req)
            });
        let user_id = i64::try_from(user_id)?;
        let req = Request::new(RegisterUserRequest { user_id, user_name });
        let response = users_client.register_user(req).await?;
        let user_role = response.into_inner().user_role.into();
        Ok(user_role)
    }
    pub async fn get_user(&self, user_id: u64) -> Result<shared::models::User> {
        let user_id = i64::try_from(user_id)?;
        let channel = tonic::transport::Channel::from_static(ADDRESS)
            .connect()
            .await?;
        let mut users_client =
            UsersServiceClient::with_interceptor(channel, move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert(AUTH, self.token.clone());
                Ok(req)
            });
        let req = Request::new(GetUserRequest { user_id });
        let response = users_client.get_user(req).await?;
        let user = response.into_inner().into();
        Ok(user)
    }
    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<shared::models::User>> {
        let channel = tonic::transport::Channel::from_static(ADDRESS)
            .connect()
            .await?;
        let mut users_client =
            UsersServiceClient::with_interceptor(channel, move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert(AUTH, self.token.clone());
                Ok(req)
            });
        let req = Request::new(ListAllUsersRequest { limit, offset });
        let response = users_client.list_all_users(req).await?;
        let users = response
            .into_inner()
            .users
            .into_iter()
            .map(shared::models::User::from)
            .collect();
        Ok(users)
    }
}
