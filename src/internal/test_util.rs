use crate::internal::config::database::{Database, DatabaseParams};

pub async fn get_conn_with_new_db() -> Database {
    let random_db_name = uuid::Uuid::now_v7().to_string();
    let random_db_name = "random_".to_string() + &random_db_name.replace("-", "_");
    println!("Random Db Name {}", random_db_name);

    let database = Database::build_with_new_database(DatabaseParams {
        host: "localhost".to_string(),
        password: "pass".to_string(),
        port: 5432,
        user: "user".to_string(),
        db_name: random_db_name,
    })
    .await
    .expect("Failed to connect to Database");
    database
}
