use axum::{
    Json, extract::{Request, State}, http::{HeaderMap, StatusCode, header}, response::{Html, IntoResponse, Response}
};

use tokio::task::spawn_blocking;

use crate::{
    sessions::{create_session, check_session},
    types::AppState,
};

use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct LoginBody {
    pub password: String,
}

#[derive(Serialize, Clone)]
pub struct Message {
    pub message: String,
}

impl Message {
    pub fn new(message: &str) -> Message {
        Message {
            message: String::from(message),
        }
    }
}

pub fn internal_error<E: std::error::Error>(err: E) -> Response {
    tracing::error!(error = %err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Message::new("Internal server error")),
    )
        .into_response()
}

pub async fn serve_website() -> Html<&'static str> { 
    Html(include_str!("./website.html"))
}

pub async fn get_favicon() -> impl IntoResponse {
    let bytes = include_bytes!("./favicon.ico");

    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        bytes
    )
}

pub async fn get_font() -> impl IntoResponse {
    let bytes = include_bytes!("./roboto.ttf");

    (
        [(header::CONTENT_TYPE, "font/ttf")],
        bytes
    )
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Response, Response> {
    let login_error = (
        StatusCode::UNAUTHORIZED,
        Json(Message::new("Invalid credentials")),
    );
    
    let password_hash = std::env::var("PASSWORD_HASH").unwrap();

    let result = spawn_blocking(move || bcrypt::verify(&body.password, &password_hash))
        .await
        .map_err(|e| internal_error(e))?
        .map_err(|e| {
            tracing::error!(error = %e, "Error while verifying password");
            login_error.clone().into_response()
        })?;

    if !result {
        return Ok(login_error.into_response());
    }

    let mut response = (StatusCode::OK, Json(Message::new("Login successful"))).into_response();

    let cookie = create_session(&state.db)
        .await
        .map_err(|e| internal_error(e))?;

    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.parse().unwrap());

    tracing::info!("User logged in");

    Ok(response)
}

fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?;
    let cookie_string = cookie_header.to_str().ok()?;

    for cookie in cookie_string.split(";") {
        if let Some(value) = cookie.trim().strip_prefix("session_id=") {
            return Some(value.to_string());
        }
    }

    None
}

pub async fn check_login(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, Response> {
    let session_id = extract_session_id(request.headers());
    let unauthorized_error =
        Ok((StatusCode::UNAUTHORIZED, Json(Message::new("Unauthorized"))).into_response());

    let session_id = match session_id {
        Some(id) => id,
        None => return unauthorized_error,
    };

    let cookie = match check_session(&state.db, &session_id)
        .await
        .map_err(|e| internal_error(e))?
    {
        Some(cookie) => cookie,
        None => return unauthorized_error,
    };

    let mut response = (StatusCode::OK, Json(Message::new("Checked login successfully"))).into_response();

    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.parse().unwrap());

    tracing::info!("User checked login");

    Ok(response)
}

