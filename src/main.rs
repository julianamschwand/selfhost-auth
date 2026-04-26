mod db;
mod handlers;
mod sessions;
mod types;

use dotenv::dotenv;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::{Router, routing};

use db::{create_pool, init_db};
use types::AppState;
use handlers::{check_login, login};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(LevelFilter::INFO)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let password = args.get(1);

    if let Some(password) = password {
        hash_password(password);
    }

    std::env::var("PASSWORD_HASH").expect(
        "PASSWORD_HASH env var has to be set. Enter your password as a parameter to the program to hash it"
    );

    init_db().await.expect("Error while initializing DB");

    let router = get_router().await;

    let port = std::env::var("PORT").unwrap_or_else(|_| String::from("3000"));
    let address = format!("127.0.0.1:{port}");

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("Server listening on port {port}");
    axum::serve(listener, router).await.unwrap();
}

pub async fn get_router() -> Router {
    let pool = create_pool().await.expect("Error while connecting DB pool");

    let state = AppState { db: pool };

    Router::new()
        .route("/login", routing::post(login))
        .route("/check-login", routing::get(check_login))
        .with_state(state)
}

pub fn hash_password(password: &str) {
    let password_hash = bcrypt::hash(password, 10).expect("Error while hashing password");
    println!("{password_hash}");
    std::process::exit(0);
}
