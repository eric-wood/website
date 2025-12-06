use std::{
    collections::HashMap,
    env,
    fs::{self, File, read_to_string},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use comrak::{markdown_to_html_with_plugins, options, plugins::syntect::SyntectAdapterBuilder};

use crate::date_time::DateTime;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BlogPost {
    pub title: Option<String>,
    pub published_at: Option<DateTime>,
    #[serde(skip_deserializing)]
    pub file_path: PathBuf,
}

pub fn get_posts() -> anyhow::Result<HashMap<String, BlogPost>> {
    let path_str = env::var("BLOG_POSTS_PATH").expect("BLOG_POSTS_PATH not set");
    let root_path = Path::new(&path_str);
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
        let mut slug = file_name.replace("_", "-");
        slug.replace_last(".md", "");
        let path = root_path.join(file_name).clone();

        let maybe_post = extract_frontmatter(&path)?;

        if let Some(mut post) = maybe_post {
            post.file_path = path;
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

pub fn render_post(path: &PathBuf) -> anyhow::Result<String> {
    let md = read_to_string(path)?;
    let adapter = SyntectAdapterBuilder::new()
        .theme("base16-eighties.dark")
        .build();
    let mut plugins = options::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);
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
