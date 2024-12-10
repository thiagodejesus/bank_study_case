use sqlx::postgres::PgPoolOptions;

#[derive(Debug)]
pub struct ConfigurationError {
    message: String,
}

pub struct DatabaseParams {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub db_name: String,
}

pub struct Database {
    pool: sqlx::PgPool,
    host: String,
    port: u16,
    user: String,
    password: String,
    db_name: String,
}

impl Database {
    pub async fn build(params: DatabaseParams) -> Result<Self, ConfigurationError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!(
                "postgres://{user}:{password}@{host}:{port}/{db_name}",
                user = &params.user,
                password = &params.password,
                host = &params.host,
                port = &params.port,
                db_name = &params.db_name
            ))
            .await;

        match pool {
            Ok(pool) => Ok(Self {
                pool,
                host: params.host,
                port: params.port,
                user: params.user,
                password: params.password,
                db_name: params.db_name,
            }),
            Err(e) => Err(ConfigurationError {
                message: e.to_string(),
            }),
        }
    }

    pub async fn build_with_new_database(
        params: DatabaseParams,
    ) -> Result<Self, ConfigurationError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!(
                "postgres://{user}:{password}@{host}:{port}/postgres",
                user = &params.user,
                password = &params.password,
                host = &params.host,
                port = &params.port,
            ))
            .await;

        match pool {
            Ok(pool) => {
                sqlx::query(&format!("CREATE DATABASE {}", &params.db_name))
                    .execute(&pool)
                    .await
                    .expect("Failed to create database");

                let pool = PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&format!(
                        "postgres://{user}:{password}@{host}:{port}/{db_name}",
                        user = &params.user,
                        password = &params.password,
                        host = &params.host,
                        port = &params.port,
                        db_name = &params.db_name
                    ))
                    .await
                    .expect("Failed to connect to new database");

                // Gets the current directory
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("Failed to run migrations");

                Ok(Self {
                    pool,
                    host: params.host,
                    port: params.port,
                    user: params.user,
                    password: params.password,
                    db_name: params.db_name,
                })
            }
            Err(e) => Err(ConfigurationError {
                message: e.to_string(),
            }),
        }
    }

    pub fn get_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
