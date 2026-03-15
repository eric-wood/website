use crate::{templates::render, views::View};
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Contents {
    Single(String),
    Many(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct ResumeSubsection {
    title: String,
    subtitle: Option<String>,
    timeframe: Option<String>,
    contents: Option<Contents>,
}

#[derive(Serialize, Deserialize)]
pub struct ResumeSection {
    title: String,
    contents: Vec<ResumeSubsection>,
}

#[derive(Serialize, Deserialize)]
pub struct ResumeData {
    email: String,
    location: String,
    sections: Vec<ResumeSection>,
}

pub struct Resume {
    data: ResumeData,
}

impl Resume {
    pub fn new(data: ResumeData) -> Self {
        Self { data }
    }
}

impl View for Resume {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let html = render(
            reloader,
            "views/resume",
            context! {
                resume => self.data,
            },
        )?;

        Ok(html)
    }
}
