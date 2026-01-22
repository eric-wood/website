use minijinja_autoreload::AutoReloader;

pub mod blog;
pub mod photos;
pub mod projects;

pub trait View {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String>;
}
