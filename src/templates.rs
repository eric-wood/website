use minijinja::{Environment, Error, ErrorKind};
use minijinja_autoreload::AutoReloader;
use serde::Serialize;
use std::{env, fs::read_to_string, path::Path};

//#[derive(Serialize)]
//struct NavLink<'a> {
//    id: &'a str,
//    label: &'a str,
//    href: &'a str,
//}

pub fn load_templates_dyn() -> AutoReloader {
    AutoReloader::new(move |notifier| {
        let mut env = Environment::new();
        env.set_loader(loader);

        notifier.set_fast_reload(true);

        let should_autoreload = env::var("AUTO_RELOAD_TEMPLATES").is_ok_and(|c| c == "true");
        if should_autoreload {
            let template_path = Path::new("templates");
            notifier.watch_path(template_path, true);
        }
        env.add_function("url_escape", url_escape);
        //env.add_global(
        //    "nav_links",
        //    [NavLink {
        //        id: "photos",
        //        label: "Photos",
        //        href: "/photos",
        //    }],
        //);
        Ok(env)
    })
}

fn loader(name: &str) -> Result<Option<String>, Error> {
    let root_path = Path::new("templates");
    let template_path = root_path.join(format!("{name}.jinja"));
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

pub fn render<S>(reloader: &AutoReloader, name: &str, context: S) -> anyhow::Result<String>
where
    S: Serialize,
{
    let template_env = reloader.acquire_env()?;
    let template = template_env.get_template(name)?;
    Ok(template.render(context)?)
}

fn url_escape(input: String) -> String {
    urlencoding::encode(&input).into_owned()
}
