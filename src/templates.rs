use chrono::Datelike;
use minijinja::{Environment, Error, ErrorKind, Value};
use minijinja_autoreload::AutoReloader;
use serde::Serialize;
use std::{fs::read_to_string, path::Path};

use crate::{config::Config, date_time::DateTime};

#[derive(Serialize)]
struct NavLink<'a> {
    id: &'a str,
    label: &'a str,
    href: &'a str,
}

pub fn load_templates_dyn(config: &Config) -> AutoReloader {
    let should_autoreload = config.auto_reload_templates;
    let content_path_str = config.content_path.clone();
    let is_prod = config.is_prod();

    AutoReloader::new(move |notifier| {
        let mut env = Environment::new();
        env.set_loader(loader);
        env.set_debug(!is_prod);

        notifier.set_fast_reload(true);

        if should_autoreload {
            let template_path = Path::new("templates");
            notifier.watch_path(template_path, true);

            let views_path = Path::new("src/views");
            notifier.watch_path(views_path, true);

            let content_path = Path::new(&content_path_str);
            notifier.watch_path(content_path, true);
        }
        env.add_function("url_escape", url_escape);
        env.add_function("inline_style", inline_style);
        env.add_function("inline_script", inline_script);
        env.add_function("photo_thumbnail_url", photo_thumbnail_url);
        env.add_function("photo_url", photo_url);
        env.add_function("assets_path", assets_path);
        env.add_function("year", year);
        env.add_function("timestamp", timestamp);
        env.add_global(
            "nav_links",
            Value::from_serialize([
                NavLink {
                    id: "projects",
                    label: "Projects",
                    href: "/projects",
                },
                NavLink {
                    id: "blog",
                    label: "Blog",
                    href: "/blog",
                },
                NavLink {
                    id: "photos",
                    label: "Photos",
                    href: "/photos",
                },
                NavLink {
                    id: "music",
                    label: "Music",
                    href: "/music",
                },
            ]),
        );
        env.add_global(
            "themes",
            Value::from_serialize([
                ("black", "white", None),
                ("white", "black", Some(16)),
                ("firebrick", "whitesmoke", None),
                ("lightgreen", "darkslategray", None),
                ("pink", "rebeccapurple", None),
                ("midnightblue", "aliceblue", None),
            ]),
        );

        Ok(env)
    })
}

fn loader(name: &str) -> Result<Option<String>, Error> {
    let is_view = name.starts_with("views");
    let has_extension = name.ends_with(".jinja");
    let root_path = Path::new(if is_view { "src" } else { "templates" });

    let mut template_path = root_path.join(name);
    if is_view && !has_extension {
        template_path.push("template")
    }
    if !has_extension {
        template_path.add_extension("jinja");
    }

    let template = read_to_string(&template_path).map_err(|_| {
        Error::new(
            ErrorKind::TemplateNotFound,
            format!(
                "Unable to locate {name} at {}",
                template_path.to_owned().to_str().unwrap()
            ),
        )
    })?;
    Ok(Some(template))
}

pub fn render<S>(
    reloader: &AutoReloader,
    name: &str,
    context: S,
) -> Result<String, minijinja::Error>
where
    S: Serialize,
{
    let template_env = reloader.acquire_env()?;
    let template = template_env.get_template(name)?;
    template.render(context)
}

fn url_escape(input: String) -> String {
    urlencoding::encode(&input).into_owned()
}

fn inline_style(path: String) -> String {
    let styles = read_to_string(path).expect("unable to locate stylesheet");
    format!("<style type=\"text/css\">\n{styles}\n</style>")
}

fn inline_script(path: String) -> String {
    let script = read_to_string(path).expect("unable to locate script");
    format!("<script type=\"text/javascript\">\n{script}\n</script>")
}

fn photo_thumbnail_url(photo_id: String) -> String {
    format!("/photos/thumbnails/{photo_id}.webp")
}

fn photo_url(filename: String) -> String {
    format!("/photos/images/{filename}")
}

fn assets_path(path: String) -> String {
    format!("/assets/{path}")
}

fn year() -> String {
    let year = chrono::Utc::now().year();
    format!("{year}")
}

fn timestamp() -> String {
    let now = DateTime::now();
    let time_str: String = now.into();
    format!("<time datetime=\"{time_str}\" data-time=\"true\">{time_str}</time>")
}
