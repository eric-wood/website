use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
    fmt::{self, Write},
    fs::{self, File, read_to_string},
    io::{BufRead, BufReader, Write as IoWrite},
    path::{Path, PathBuf},
    sync::Arc,
};

use arborium::Highlighter;
use comrak::{
    Arena,
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    arena_tree::NodeEdge,
    html::format_document_with_plugins,
    nodes::{AstNode, NodeValue, Sourcepos},
    options, parse_document,
};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    AppState,
    config::Config,
    date_time::DateTime,
    views::{View, blog::BlogShow},
};

pub struct BlogStore {
    by_slug: HashMap<String, Arc<BlogPost>>,
    by_tag: HashMap<String, Vec<Arc<BlogPost>>>,
}

impl BlogStore {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let root_path = Path::new(&config.blog_posts_path);
        let mut by_slug = HashMap::new();
        let mut by_tag: HashMap<String, Vec<Arc<BlogPost>>> = HashMap::new();

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
                post.slug = Some(slug.clone());
                post.file_path = path;
                post.cache_path = Path::new(&config.cache_path).join("blog").join(&slug);
                let post = Arc::new(post);
                by_slug.insert(slug, post.clone());

                for tag in post.tags.iter() {
                    if let Some(tag_posts) = by_tag.get_mut(tag) {
                        tag_posts.push(post.clone());
                    } else {
                        by_tag.insert(tag.clone(), vec![post.clone()]);
                    }
                }
            }
        }

        Ok(Self { by_slug, by_tag })
    }

    pub fn all(&self) -> Vec<Arc<BlogPost>> {
        self.by_slug.values().cloned().collect()
    }

    pub fn all_tags(&self) -> Vec<(String, usize)> {
        let mut tags: Vec<(String, usize)> = self
            .by_tag
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));

        tags
    }

    pub fn get_by_slug(&self, slug: &str) -> Option<Arc<BlogPost>> {
        self.by_slug.get(slug).cloned()
    }

    pub fn get_by_tag(&self, tag: &str) -> Vec<Arc<BlogPost>> {
        self.by_tag
            .get(tag)
            .unwrap_or(&Vec::<Arc<BlogPost>>::new())
            .to_vec()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct BlogPost {
    pub title: Option<String>,
    pub published_at: Option<DateTime>,
    pub slug: Option<String>,
    #[serde(deserialize_with = "deserialize_tags")]
    pub tags: Vec<String>,
    #[serde(skip_deserializing)]
    pub file_path: PathBuf,
    #[serde(skip_deserializing)]
    pub cache_path: PathBuf,
}

fn deserialize_tags<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer).unwrap_or_default();
    Ok(str_sequence
        .replace(", ", ",")
        .split(',')
        .map(|i| i.to_owned())
        .collect())
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

        if lang.is_empty() {
            output.write_str(code)?;
            return Ok(());
        }

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
            "</section>\n<section id=\"{}\" class=\"blog__section\"><div class=\"blog__heading\">\n<h{}>",
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

fn text_from_node<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.children() {
        if let NodeValue::Text(s) = &child.data.borrow().value {
            text.push_str(s.borrow());
        } else {
            text.push_str(&text_from_node(child));
        }
    }

    text
}

#[derive(Serialize, Clone, Debug)]
pub struct Section {
    pub name: String,
    pub slug: String,
    pub level: u8,
    pub subsections: Vec<Section>,
}

fn create_toc<'a>(root: &'a AstNode<'a>) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();

    for edge in root.traverse() {
        if let NodeEdge::Start(node) = edge
            && let NodeValue::Heading(ref heading) = node.data().value
        {
            let name = text_from_node(node);
            let slug = slug::slugify(&name);
            let level = heading.level;

            let section = Section {
                name,
                slug,
                level,
                subsections: Vec::new(),
            };
            sections.push(section);
        }
    }

    let mut toc: VecDeque<Section> = VecDeque::new();
    let mut queue: VecDeque<Section> = VecDeque::from(sections);
    while let Some(mut current) = queue.pop_back() {
        while let Some(section) = toc.front()
            && section.level > current.level
        {
            let section = toc.pop_front().unwrap();
            current.subsections.push(section);
        }

        toc.push_front(current);
    }

    toc.into()
}

fn fix_sections(body: &str) -> String {
    let re = Regex::new(r"<\/?section>").unwrap();
    let mut result = body.to_string();
    for (i, m) in re.find_iter(body).enumerate() {
        if m.as_str() == "<section>" && i == 0 {
            break;
        }
        if m.as_str() == "</section>" && i == 0 {
            result = body.replacen("</section>", "", 1);
            break;
        }
    }

    let close = "</section>";
    if let Some(i) = body.rfind("<section class=\"footnotes\"") {
        result.insert_str(i - close.len(), close);
    } else {
        result += close;
    }

    result
}

pub fn render_post(path: &PathBuf) -> anyhow::Result<(String, Vec<Section>)> {
    let md = read_to_string(path)?;
    let syntax_adapter = SyntaxAdapter::new();
    let heading_adapter = LinkedHeadingAdapter;
    let mut plugins = options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&syntax_adapter);
    plugins.render.heading_adapter = Some(&heading_adapter);
    let options = comrak::Options {
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
    };
    let arena = Arena::new();
    let root = parse_document(&arena, &md, &options);
    let toc = create_toc(root);

    let mut body = String::new();
    format_document_with_plugins(root, &options, &mut body, &plugins)?;
    body = fix_sections(&body);

    Ok((body, toc))
}

// Render all of our blog posts as fully static HTML files so we can serve them up without having
// to do all that extra markdown stuff. Maybe find a better way to share this logic with the
// controller in the near future.
pub fn cache_posts(state: &AppState) -> anyhow::Result<()> {
    if !state.config.is_prod() {
        return Ok(());
    }

    for post in state.blog_store.all() {
        let view = BlogShow::new(post.clone());
        let rendered = view.render(&state.reloader)?;

        let mut file = File::create(post.cache_path.clone())?;
        file.write_all(rendered.as_bytes())?;
    }

    Ok(())
}
