use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

pub fn get_posts() -> anyhow::Result<HashMap<String, PathBuf>> {
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
        map.insert(slug, path);
    }
    Ok(map)
}
