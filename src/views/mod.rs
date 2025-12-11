use minijinja_autoreload::AutoReloader;

pub mod blog;
pub mod photos;

pub trait View {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String>;
}

//
// views
//   - photos
//      - show
//          - show.jinja
//          - show.rs
//          - show.js
//          - show.css
//      - index
