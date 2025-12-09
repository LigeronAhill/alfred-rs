use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};

const NAMESPACE: &str = "alfred";
const DATABASE: &str = "users";

pub struct UsersStorage {
    client: Surreal<Client>,
}

impl UsersStorage {
    pub async fn init(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> surrealdb::Result<Self> {
        let addr = format!("{}:{}", host, port);
        let client = Surreal::new::<Ws>(&addr).await?;
        client.signin(Root { username, password }).await?;
        client.use_ns(NAMESPACE).use_db(DATABASE).await?;
        Ok(Self { client })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_failed() {
        let host = "localhost";
        let port = 8000;
        let username = "user";
        let password = "password";

        let result = UsersStorage::init(host, port, username, password)
            .await
            .inspect_err(|e| {
                println!("{e}");
            });

        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_init_success() {
        let host = std::env::var("DB_HOST").expect("DB_HOST not set");
        let port = std::env::var("DB_PORT")
            .expect("DB_PORT not set")
            .parse()
            .expect("DB_PORT not a number");
        let username = std::env::var("DB_USER").expect("DB_USER not set");
        let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD not set");

        let result = UsersStorage::init(&host, port, &username, &password)
            .await
            .inspect_err(|e| {
                dbg!(e);
            });

        assert!(result.is_ok());
    }
}
