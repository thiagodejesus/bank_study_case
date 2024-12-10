use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Create a connection pool
    //  for MySQL/MariaDB, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.

    println!("Will open the conn");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:pass@localhost:5432/db")
        .await?;

    println!("Will Select");
    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await?;

    println!("Row result: {:?}", row);

    assert_eq!(row.0, 150);

    Ok(())
}
