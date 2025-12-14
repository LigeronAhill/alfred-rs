pub mod middleware;
mod routes;
use std::sync::Arc;

pub const TOKEN: &str = "alfred-token";

use serde::{Deserialize, Serialize};

use crate::{
    AppError, AppResult,
    services::UsersService,
    settings::{JWTSettings, ServerSettings},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub struct Server {
    addr: String,
    origin: String,
    state: Arc<AppState>,
}
impl Server {
    pub fn new(settings: ServerSettings, state: Arc<AppState>) -> Self {
        Self {
            addr: settings.server_address(),
            origin: settings.origin,
            state,
        }
    }
    pub async fn start(&self) -> AppResult<()> {
        tracing::info!("Starting server on {addr}", addr = self.addr);
        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        tracing::info!("Server listening on {addr}", addr = self.addr);
        let app = routes::init(self.state.clone(), &self.origin);
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
        {
            return Err(AppError::IOError(e));
        }
        tracing::info!("Server shutting down gracefully");
        Ok(())
    }
}

#[derive(Clone)]
pub struct AppState {
    pub users_service: Arc<UsersService>,
    pub jwt_settings: Arc<JWTSettings>,
}
impl AppState {
    pub fn new(users_service: Arc<UsersService>, jwt_settings: Arc<JWTSettings>) -> Self {
        Self {
            users_service,
            jwt_settings,
        }
    }
}
async fn shutdown_signal() {
    tracing::info!("Shutdown signal handler installed");
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}
