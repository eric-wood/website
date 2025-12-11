use minijinja::context;
use minijinja_autoreload::AutoReloader;

use crate::{Photo, models::Tag, templates::render, views::View};

pub struct PhotosShow {
    photo: Photo,
    tags: Vec<Tag>,
}

impl PhotosShow {
    pub fn new(photo: Photo, tags: Vec<Tag>) -> Self {
        Self { photo, tags }
    }
}

impl View for PhotosShow {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let aperture = format!("{:.1}", self.photo.aperture);
        let focal_length = format!("{:.0}", self.photo.focal_length);
        let shutter_speed = format!("1/{:.0}s", 1.0 / self.photo.shutter_speed);

        let html = render(
            reloader,
            "views/photos/show",
            context! {
                photo => self.photo,
                tags => self.tags,
                aperture,
                focal_length,
                shutter_speed,
            },
        )?;

        Ok(html)
    }
}
