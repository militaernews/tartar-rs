use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct AppError {
    pub code: StatusCode,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("AppError: {}", self.message);
        (self.code, self.message ).into_response()
    }
}

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

}

macro_rules! from_error {
    ($err_type:ty, $err_msg:expr) => {
        impl From<$err_type> for AppError {
            fn from(err: $err_type) -> Self {
                AppError::new(format!($err_msg, err))
            }
        }
    };
}
from_error!(sqlx::Error, "Database query error: {:#}");
from_error!(String, "{}");
from_error!(&str, "{}");