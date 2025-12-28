use std::sync::Arc;

use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{blog::BlogPost, templates::render, views::View};

pub struct BlogIndex {
    posts: Vec<Arc<BlogPost>>,
}

impl BlogIndex {
    pub fn new(posts: Vec<Arc<BlogPost>>) -> Self {
        Self { posts }
    }
}

impl View for BlogIndex {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let html = render(
            reloader,
            "views/blog/index",
            context! {
                posts => self.posts
            },
        )?;

        Ok(html)
    }
}
