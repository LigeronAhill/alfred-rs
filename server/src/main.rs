use anyhow::Result;
use tonic::transport::Server;

mod auth;

mod users;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_file(true)
        .compact()
        .init();
    let addr = "[::1]:50051".parse()?;
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::users::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    let reflection_service_alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::users::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;
    let users_server = users::UsersServer::new(database::UsersStorage::new().await?);
    let users_service = proto::users::users_service_server::UsersServiceServer::with_interceptor(
        users_server,
        auth::check_auth,
    );
    tracing::info!("Starting server at {addr:?}");
    Server::builder()
        .trace_fn(|_| tracing::info_span!("alfred_server"))
        .add_service(reflection_service)
        .add_service(reflection_service_alpha)
        .add_service(users_service)
        .serve(addr)
        .await?;
    Ok(())
}
