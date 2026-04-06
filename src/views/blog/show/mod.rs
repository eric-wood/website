use std::sync::Arc;

use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{
    post::{Post, Section, render_post},
    templates::render,
    views::View,
};

pub struct BlogShow {
    post: Arc<Post>,
    assets_path: String,
}

impl BlogShow {
    pub fn new(post: Arc<Post>, assets_path: String) -> Self {
        Self { post, assets_path }
    }
}

impl View for BlogShow {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let (body, toc) = render_post(&self.post.file_path, &self.assets_path)?;
        let has_code = body.contains("<pre class=\"highlighted\">");
        let toc_html = render_toc(toc);
        let html = render(
            reloader,
            "views/blog/show",
            context! {
                post => self.post,
                body,
                toc_html,
                has_code,
            },
        )?;

        Ok(html)
    }
}

fn render_toc(toc: Vec<Section>) -> String {
    if toc.is_empty() {
        return "".to_string();
    }

    let mut markup = vec!["<ul>".to_string()];
    for section in toc {
        let Section {
            name,
            slug,
            subsections,
            ..
        } = section;
        let subsections = render_toc(subsections);
        markup.push(format!(
            "<li><a href=\"#{slug}\">{name}</a>\n{subsections}</li>"
        ));
    }

    markup.push("</ul>".to_string());
    markup.join("\n")
}
