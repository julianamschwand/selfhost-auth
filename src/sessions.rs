use rand::RngExt;
use sqlx::SqlitePool;

fn generate_session_id() -> String {
    rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn time_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn create_cookie(session_id: &str) -> String {
    let domain = std::env::var("COOKIE_DOMAIN").unwrap();
    let mut cookie = format!("session_id={session_id}; HttpOnly; Path=/; Domain={domain} SameSite=Lax; Max-Age=2592000");

    if let Ok(app_env) = std::env::var("APP_ENV") {
        if app_env == "production" {
            cookie.push_str("; Secure");
        }
    }

    cookie
}

pub async fn create_session(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let session_id = generate_session_id();

    let expires_at = time_now() + (60 * 60 * 24 * 30); // 30 days

    sqlx::query!(
        "INSERT INTO sessions (session_id, expires_at) values (?,?)",
        session_id,
        expires_at
    )
    .execute(pool)
    .await?;

    let cookie = create_cookie(&session_id);

    Ok(cookie)
}

pub async fn check_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let now = time_now();

    sqlx::query!("DELETE FROM sessions WHERE ? > expires_at", now)
        .execute(pool)
        .await?;

    let session = sqlx::query!(
        "SELECT session_id FROM sessions WHERE session_id = ?",
        session_id
    )
    .fetch_optional(pool)
    .await?;

    let cookie = create_cookie(&session_id);
    let expires_at = time_now() + (60 * 60 * 24 * 30); // 30 days

    match session {
        Some(_) => {
            sqlx::query!(
                "UPDATE sessions SET expires_at = ? where session_id = ?",
                expires_at,
                session_id
            )
            .execute(pool)
            .await?;
            
            Ok(Some(cookie))
        },
        None => Ok(None),
    }
}
