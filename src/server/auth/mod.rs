use std::sync::Arc;

use poem::{
    IntoResponse, Route, handler,
    http::StatusCode,
    post,
    web::{Data, Json},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    AppState,
    database::data::{Credentials, User},
};

#[derive(Deserialize)]
pub struct SignupRequestData {
    pub email: String,
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequestData {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}

#[handler]
async fn signup(
    data: Json<SignupRequestData>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<impl IntoResponse> {
    let user = User::new(&data.name);
    let creds = Credentials::new(user.id, &data.email, &data.username)
        .add_password_and_salt(data.password.as_bytes())
        .map_err(|e| {
            poem::Error::from_string(
                format!("Failed to hash password: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    state.database.create_user(user, creds).await.map_err(|e| {
        poem::Error::from_string(
            format!("Failed to create user: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    Ok((StatusCode::CREATED, Json(json!({}))))
}

#[handler]
async fn login(
    data: Json<LoginRequestData>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<impl IntoResponse> {
    let session_id = state
        .authenticate(&data.username, &data.email, &data.password)
        .await
        .map_err(|e| {
            poem::Error::from_string(
                format!("Failed to authenticate: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!(
            {
                "session_id": session_id
            }
        )),
    ))
}

pub fn route() -> Route {
    Route::new()
        .at("/signin", post(login))
        .at("/signup", post(signup))
}
