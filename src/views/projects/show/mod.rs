use std::sync::Arc;

use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{
    post::{Post, render_post},
    templates::render,
    views::View,
};

pub struct ProjectsShow {
    post: Arc<Post>,
}

impl ProjectsShow {
    pub fn new(post: Arc<Post>) -> Self {
        Self { post }
    }
}

impl View for ProjectsShow {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let (body, _) = render_post(&self.post.file_path)?;
        let has_code = body.contains("<pre class=\"highlighted\">");
        let html = render(
            reloader,
            "views/projects/show",
            context! {
                post => self.post,
                body,
                has_code,
            },
        )?;

        Ok(html)
    }
}
