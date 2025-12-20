use std::{
    collections::HashMap,
    fmt::{self, Write},
    fs::{self, File, read_to_string},
    io::{BufRead, BufReader, Write as IoWrite},
    path::{Path, PathBuf},
};

use arborium::Highlighter;
use comrak::{
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    markdown_to_html_with_plugins,
    nodes::Sourcepos,
    options,
};
use minijinja::context;

use crate::{AppState, config::Config, date_time::DateTime, templates::render};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct BlogPost {
    pub title: Option<String>,
    pub published_at: Option<DateTime>,
    #[serde(skip_deserializing)]
    pub file_path: PathBuf,
    #[serde(skip_deserializing)]
    pub cache_path: PathBuf,
}

#[derive(Clone)]
struct SyntaxAdapter {}

impl SyntaxAdapter {
    pub fn new() -> Self {
        SyntaxAdapter {}
    }
}

impl SyntaxHighlighterAdapter for SyntaxAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn std::fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::fmt::Result {
        if lang.is_none() {
            output.write_str(code)?;
            return Ok(());
        }
        let lang = lang.unwrap();

        let mut highlighter = Highlighter::new();
        let html = highlighter
            .highlight(lang, code)
            .map_err(|_| std::fmt::Error)?;
        output.write_str(&html)
    }

    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: HashMap<&'static str, std::borrow::Cow<'s, str>>,
    ) -> std::fmt::Result {
        if attributes.contains_key("lang") {
            write!(
                output,
                "<pre class=\"highlighted\" data-language=\"{}\">",
                attributes["lang"]
            )
        } else {
            output.write_str("<pre class=\"highlighted\">")
        }
    }

    fn write_code_tag<'s>(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: HashMap<&'static str, std::borrow::Cow<'s, str>>,
    ) -> std::fmt::Result {
        // why doesn't it give us the actual language string????? hmmm????
        if attributes.contains_key("class") {
            let lang = attributes["class"].replace("language-", "");
            write!(output, "<code data-language=\"{}\">", lang)
        } else {
            output.write_str("<code>")
        }
    }
}

pub fn load_index(config: &Config) -> anyhow::Result<HashMap<String, BlogPost>> {
    let root_path = Path::new(&config.blog_posts_path);
    let mut map = HashMap::new();

    let names: Vec<String> = fs::read_dir(root_path)?
        .filter_map(|i| i.ok())
        .filter_map(|entry| {
            let file_name = entry.file_name().into_string().unwrap_or("".to_string());
            if !file_name.ends_with("md") {
                return None;
            }

            Some(file_name)
        })
        .collect();

    for file_name in names {
        let mut slug = slug::slugify(&file_name);
        slug.replace_last("-md", "");
        let path = root_path.join(file_name).clone();

        let maybe_post = extract_frontmatter(&path)?;

        if let Some(mut post) = maybe_post {
            post.file_path = path;
            post.cache_path = Path::new(&config.cache_path).join("blog").join(&slug);
            map.insert(slug, post);
        }
    }
    Ok(map)
}

fn extract_frontmatter(path: &PathBuf) -> anyhow::Result<Option<BlogPost>> {
    let file = File::open(path)?;

    let mut yaml = String::new();
    let mut lines = BufReader::new(file).lines();

    let first_line = lines.next().unwrap_or(Ok("".to_string()))?;
    if first_line != "---" {
        return Ok(None);
    }

    for line in lines {
        if line.is_err() {
            break;
        }

        let line = line.unwrap_or("".to_string());

        if line == "---" {
            break;
        }

        yaml.push_str(&format!("\n{line}"));
    }

    let post: BlogPost = serde_yaml::from_str(&yaml)?;

    Ok(Some(post))
}

struct LinkedHeadingAdapter;

impl HeadingAdapter for LinkedHeadingAdapter {
    fn enter(
        &self,
        output: &mut dyn Write,
        heading: &HeadingMeta,
        _sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        let id = slug::slugify(&heading.content);

        write!(
            output,
            "<div class=\"blog__heading\" id=\"{}\">\n<h{}>",
            id, heading.level
        )
    }

    fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> fmt::Result {
        let id = slug::slugify(&heading.content);
        write!(
            output,
            "</h{}><a href=\"#{}\" aria-label=\"Permalink: {}\">#</a></div>",
            heading.level, id, heading.content
        )
    }
}

pub fn render_post(path: &PathBuf) -> anyhow::Result<String> {
    let md = read_to_string(path)?;
    let syntax_adapter = SyntaxAdapter::new();
    let heading_adapter = LinkedHeadingAdapter;
    let mut plugins = options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&syntax_adapter);
    plugins.render.heading_adapter = Some(&heading_adapter);
    let body = markdown_to_html_with_plugins(
        &md,
        &comrak::Options {
            extension: options::Extension::builder()
                .math_dollars(true)
                .multiline_block_quotes(true)
                .strikethrough(true)
                .superscript(true)
                .footnotes(true)
                .underline(true)
                .greentext(true)
                .autolink(true)
                .alerts(true)
                .table(true)
                .math_code(true)
                .maybe_front_matter_delimiter(Some("---".to_string()))
                .build(),
            parse: options::Parse::builder().build(),
            render: options::Render::builder().build(),
        },
        &plugins,
    );

    Ok(body)
}

// Render all of our blog posts as fully static HTML files so we can serve them up without having
// to do all that extra markdown stuff. Maybe find a better way to share this logic with the
// controller in the near future.
pub fn cache_posts(state: &AppState) -> anyhow::Result<()> {
    if !state.config.is_prod() {
        return Ok(());
    }

    for post in state.blog_slugs.values() {
        let body = render_post(&post.file_path)?;
        let post = post.clone();
        let rendered = render(
            &state.reloader,
            "blog/show",
            context! {
                post,
                body,
            },
        )?;

        let mut file = File::create(post.cache_path)?;
        file.write_all(rendered.as_bytes())?;
    }

    Ok(())
}
