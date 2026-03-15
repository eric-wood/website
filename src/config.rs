use std::{env, path::Path};

#[derive(PartialEq)]
pub enum Environment {
    Development,
    Production,
}

pub struct Config {
    pub environment: Environment,
    pub cache_path: String,
    pub auto_reload_templates: bool,
    pub content_path: String,
    pub resume_path: String,
    pub blog_posts_path: String,
    pub projects_path: String,
    pub content_assets_path: String,
    pub photos_db_path: String,
    pub photos_thumbnail_path: String,
    pub photos_image_path: String,
    pub assets_path: String,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let environment = match env::var("ENVIRONMENT")
            .unwrap_or("development".to_string())
            .as_str()
        {
            "development" => Environment::Development,
            "production" => Environment::Production,
            _ => Environment::Development,
        };

        let cache_path = env::var("CACHE_PATH").unwrap_or(".cache".to_string());

        let auto_reload_templates = env::var("AUTO_RELOAD_TEMPLATES").is_ok_and(|c| c == "true");

        let content_path = env::var("CONTENT_PATH").expect("CONTENT_PATH env variable not set");
        let content_folder_path = Path::new(&content_path);
        let resume_path = content_folder_path
            .join("resume.yaml")
            .display()
            .to_string();
        let blog_posts_path = content_folder_path.join("blog_posts").display().to_string();
        let projects_path = content_folder_path.join("projects").display().to_string();
        let content_assets_path = content_folder_path.join("assets").display().to_string();

        let photos_db_path =
            env::var("PHOTOS_DB_PATH").expect("PHOTOS_DB_PATH env variable not set");
        let photos_thumbnail_path =
            env::var("PHOTOS_THUMBNAIL_PATH").expect("PHOTOS_THUMBNAIL_PATH env variable not set");
        let photos_image_path =
            env::var("PHOTOS_IMAGE_PATH").expect("PHOTOS_IMAGE_PATH env variable not set");
        let assets_path = env::var("ASSETS_PATH").expect("ASSETS_PATH env variable not set");

        Ok(Self {
            environment,
            cache_path,
            auto_reload_templates,
            content_path,
            resume_path,
            blog_posts_path,
            projects_path,
            content_assets_path,
            photos_db_path,
            photos_thumbnail_path,
            photos_image_path,
            assets_path,
        })
    }

    pub fn is_prod(&self) -> bool {
        self.environment == Environment::Production
    }
}
