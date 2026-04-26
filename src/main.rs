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
use tokio::signal;

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

    let port = std::env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let address = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("Server listening on port {port}");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
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

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutting down ...");
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
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).expect("Error while hashing password");
    println!("{password_hash}");
    std::process::exit(0);
}
