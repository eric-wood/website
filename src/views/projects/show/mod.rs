use std::sync::Arc;

use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{
    post::{Post, Section, render_post},
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
        let (body, toc) = render_post(&self.post.file_path)?;
        let has_code = body.contains("<pre class=\"highlighted\">");
        let toc_html = render_toc(toc);
        let html = render(
            reloader,
            "views/projects/show",
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
