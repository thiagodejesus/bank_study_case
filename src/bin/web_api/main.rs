mod account;
mod transaction;

use axum::{
    routing::{get, post},
    Router,
};
use bank_case::internal::config::database::{Database, DatabaseParams};
use std::sync::Arc;

pub struct AppState {
    pg_pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let database = Database::build(DatabaseParams {
        db_name: "db".to_string(),
        host: "localhost".to_string(),
        password: "pass".to_string(),
        user: "user".to_string(),
        port: 5432,
    })
    .await
    .expect("Failed to connect to Database");

    let pool = database.get_pool();

    let app_state = Arc::new(AppState {
        pg_pool: pool.clone(),
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        // .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/account", post(account::create_account_controller))
        .route(
            "/account/:account_number/balance",
            get(account::get_balance),
        )
        .route("/accounts", get(account::list_accounts_controller))
        .route("/transaction", post(transaction::create_transaction))
        .with_state(app_state);

    let uri = "0.0.0.0:3000";
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(uri)
        .await
        .expect("Failed to bind port");
    println!("Running server on {}", uri);

    axum::serve(listener, app)
        .await
        .expect("Failed to serve app");
}
