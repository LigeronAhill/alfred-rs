use database::UsersStorage;
use proto::users::{
    DeleteUserRequest, DeleteUserResponse, GetUserRequest, GetUserResponse, ListAllUsersRequest,
    ListAllUsersResponse, RegisterUserRequest, RegisterUserResponse, UpdateUserRoleRequest,
    UpdateUserRoleResponse,
};
use tonic::{Request, Response, Status};
use tracing::log::info;

pub struct UsersServer {
    storage: UsersStorage,
}
impl UsersServer {
    pub fn new(storage: UsersStorage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl proto::users::users_service_server::UsersService for UsersServer {
    #[tracing::instrument(name = "register user", skip(self))]
    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> tonic::Result<Response<RegisterUserResponse>> {
        info!("Received: {request:?}");
        let user_id = request.get_ref().user_id;
        let user_name = request.into_inner().user_name;
        let user_role = self
            .storage
            .register_user(user_id, user_name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let response = RegisterUserResponse {
            user_role: user_role.to_string(),
        };
        Ok(Response::new(response))
    }

    #[tracing::instrument(name = "get user by id", skip(self))]
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> tonic::Result<Response<GetUserResponse>> {
        info!("Received: {request:?}");
        let user_id = request.into_inner().user_id;
        let user = self
            .storage
            .get_user_by_id(user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or(Status::not_found("User not found!"))?;
        Ok(Response::new(user.into()))
    }
    #[tracing::instrument(name = "list all users", skip(self))]
    async fn list_all_users(
        &self,
        request: Request<ListAllUsersRequest>,
    ) -> tonic::Result<Response<ListAllUsersResponse>> {
        info!("Received: {request:?}");
        let r = request.get_ref();
        let limit = r.clone().limit;
        let offset = r.offset;
        let result = self
            .storage
            .get_all_users(limit, offset)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .into_iter()
            .map(proto::users::User::from)
            .collect();
        let response = ListAllUsersResponse { users: result };
        Ok(Response::new(response))
    }
    #[tracing::instrument(name = "update user role", skip(self))]
    async fn update_user_role(
        &self,
        request: Request<UpdateUserRoleRequest>,
    ) -> tonic::Result<Response<UpdateUserRoleResponse>> {
        info!("Received: {request:?}");
        let user = request.get_ref();
        let user_id = user.clone().user_id;
        let new_user_role = user.clone().new_user_role;
        let user = self
            .storage
            .update_user_role(user_id, new_user_role.into())
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or(Status::not_found("User not found!"))?;
        let response = UpdateUserRoleResponse {
            user_id: user.user_id,
            user_name: user.user_name,
            user_role: user.user_role.to_string(),
            created_at: user.created_at.to_string(),
            updated_at: user.updated_at.to_string(),
        };
        Ok(Response::new(response))
    }
    #[tracing::instrument(name = "delete user", skip(self))]
    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> tonic::Result<Response<DeleteUserResponse>> {
        info!("Received: {request:?}");
        let user_id = request.into_inner().user_id;
        self.storage
            .delete_user(user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteUserResponse { success: true }))
    }
}
