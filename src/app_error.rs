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
        } else if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            let body = if let Self::Anyhow(e) = &self
                && let Some(template_error) = e.downcast_ref::<minijinja::Error>()
            {
                let debug = template_error.display_debug_info().to_string();
                let kind = template_error.kind();
                let detail = template_error.detail().unwrap_or_default();
                let name = template_error.name().unwrap_or_default();
                let line = template_error.line().unwrap_or_default();

                format!("{kind}: {detail}\n{name}:{line}\n{}", debug)
            } else {
                format!("{:#?}", self)
            };

            render_error(body)
        } else {
            self.to_string()
        };

        (self.status_code(), Html(body)).into_response()
    }
}

fn render_error(body: String) -> String {
    let style = r#"
        body {
            padding: 30px;
            font-family: sans-serif;
        }

        pre {
            background-color: #eee;
            padding: 20px;
            border-radius: 6px;
        }
    "#;

    format!(
        r#"
<html>
    <head>
        <style type="text/css">{style}</style>
    </head>
    <body>
        <h1>Error:</h1>
        <pre><code>{}</code></pre>
    </body>
</html>
"#,
        html_escape::encode_text(&body)
    )
}
