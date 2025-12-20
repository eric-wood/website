use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{
    blog::{BlogPost, render_post},
    templates::render,
    views::View,
};

pub struct BlogShow<'a> {
    post: &'a BlogPost,
}

impl<'a> BlogShow<'a> {
    pub fn new(post: &'a BlogPost) -> Self {
        Self { post }
    }
}

impl<'a> View for BlogShow<'a> {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let body = render_post(&self.post.file_path)?;
        let has_code = body.contains("<pre class=\"highlighted\">");
        let html = render(
            reloader,
            "views/blog/show",
            context! {
                post => self.post,
                body,
                has_code,
            },
        )?;

        Ok(html)
    }
}
