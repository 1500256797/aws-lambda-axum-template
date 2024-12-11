use axum::{
    http,
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    status: u16,
    message: String,
    data: T,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn new(status: u16, message: String, data: T) -> ApiResponse<T> {
        ApiResponse {
            status,
            message,
            data,
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let json_body = Json(self);
        (status_code, json_body).into_response()
    }
}
