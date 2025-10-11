use anyhow::Context;
use minijinja::Environment;
use std::{
    fs::{self, DirEntry, read_to_string},
    path::Path,
};

pub fn load_templates<'a>() -> anyhow::Result<Environment<'a>> {
    let mut env = Environment::new();

    let path = Path::new("templates");
    visit_dirs(path, &mut |entry: &DirEntry| -> anyhow::Result<()> {
        let path = entry.path();
        let file_name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let parent = path.parent().unwrap();
        let prefix = parent.strip_prefix("templates")?.to_str().unwrap();
        let name = if prefix.is_empty() {
            file_name
        } else {
            format!("{prefix}/{file_name}")
        };

        let template_str = read_to_string(entry.path())
            .with_context(|| format!("read template file: {:?}", entry.path()))?;
        env.add_template_owned(name, template_str)
            .context("add_template_owned")?;

        Ok(())
    })?;

    Ok(env)
}

fn visit_dirs(
    dir: &Path,
    cb: &mut dyn FnMut(&DirEntry) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry).context("visit_dirs callback")?;
            }
        }
    }
    Ok(())
}
