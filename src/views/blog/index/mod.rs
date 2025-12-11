use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{templates::render, views::View};

pub struct BlogIndex<'a> {
    slugs: Vec<&'a String>,
}

impl<'a> BlogIndex<'a> {
    pub fn new(slugs: Vec<&'a String>) -> Self {
        Self { slugs }
    }
}

impl<'a> View for BlogIndex<'a> {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let html = render(
            reloader,
            "views/blog/index",
            context! {
                slugs => self.slugs
            },
        )?;

        Ok(html)
    }
}
