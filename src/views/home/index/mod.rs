use std::sync::Arc;

use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{models::Photo, post::Post, templates::render, views::View};

pub struct HomeIndex {
    blog_posts: Vec<Arc<Post>>,
    projects: Vec<Arc<Post>>,
    photos: Vec<Photo>,
}

impl HomeIndex {
    pub fn new(blog_posts: Vec<Arc<Post>>, projects: Vec<Arc<Post>>, photos: Vec<Photo>) -> Self {
        let blog_posts: Vec<Arc<Post>> = first_n(&blog_posts, 5);
        let projects: Vec<Arc<Post>> = first_n(&projects, 5);

        Self {
            blog_posts,
            projects,
            photos,
        }
    }
}

impl View for HomeIndex {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let html = render(
            reloader,
            "views/home/index",
            context! {
                blog_posts => self.blog_posts,
                projects => self.projects,
                photos => self.photos,
            },
        )?;

        Ok(html)
    }
}

fn first_n<T>(collection: &[T], n: usize) -> Vec<T>
where
    T: Clone,
{
    collection[..std::cmp::min(n, collection.len())].to_vec()
}
