use std::sync::Arc;

use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{post::Post, templates::render, views::View};

pub struct BlogIndex {
    posts: Vec<Arc<Post>>,
    tags: Vec<(String, usize)>,
    tag: Option<String>,
}

impl BlogIndex {
    pub fn new(posts: Vec<Arc<Post>>, tags: Vec<(String, usize)>, tag: Option<String>) -> Self {
        Self { posts, tags, tag }
    }
}

impl View for BlogIndex {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let html = render(
            reloader,
            "views/blog/index",
            context! {
                posts => self.posts,
                tags => self.tags,
                tag => self.tag,
            },
        )?;

        Ok(html)
    }
}
