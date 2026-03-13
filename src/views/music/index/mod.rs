use crate::{templates::render, views::View};
use minijinja::context;
use minijinja_autoreload::AutoReloader;

pub struct MusicIndex {}

impl MusicIndex {
    pub fn new() -> Self {
        Self {}
    }
}

impl View for MusicIndex {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let html = render(reloader, "views/music/index", context! {})?;

        Ok(html)
    }
}
