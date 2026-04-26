use sqlx::{
    ConnectOptions, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;

const DB_URL: &str = "sqlite:./data/database.sqlite3";

pub async fn init_db() -> Result<(), sqlx::Error> {
    let options = SqliteConnectOptions::from_str(DB_URL)?.create_if_missing(true);

    let mut connection = options.connect().await?;

    let schema = include_str!("schema.sql");
    sqlx::query(schema).execute(&mut connection).await?;

    Ok(())
}

pub async fn create_pool() -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(2)
        .connect(DB_URL)
        .await
}
