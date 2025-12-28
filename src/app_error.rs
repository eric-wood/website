use std::env;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("bad request")]
    ValidationError(#[from] serde_valid::validation::Errors),

    #[error(transparent)]
    DbError(#[from] sqlx::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TemplateError(#[from] minijinja::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::DbError(_) | Self::Anyhow(_) | Self::IoError(_) | Self::TemplateError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let is_dev = env::var("ENVIRONMENT").unwrap_or("development".to_string()) == "development";
        let body = if status_code == StatusCode::INTERNAL_SERVER_ERROR && !is_dev {
            "internal server error".to_string()
        } else {
            self.to_string()
        };

        (self.status_code(), Html(body)).into_response()
    }
}
