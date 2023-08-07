use std::path::{Path, PathBuf};
use egui_extras::RetainedImage;
use walkdir::WalkDir;

#[derive(Default)]
pub struct ImageSequence {
    pub directory: PathBuf,
    pub images: Vec<String>,
}

impl ImageSequence {
    pub fn new(directory: &Path) -> Self {
        Self {
            directory: PathBuf::from(directory),
            ..Default::default()
        }
    }

    pub fn reload(&mut self) {
        self.images.clear();

        let mut entries: Vec<_> = WalkDir::new(&self.directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by(|a, b| a.path().cmp(b.path()));

        for entry in entries {
            if let Some(extension) = entry.path().extension() {
                if let Some(ext_str) = extension.to_str() {
                    if ext_str == "png" || ext_str == "jpg" || ext_str == "jpeg" {
                        self.images.push(entry.path().to_str().unwrap().to_string());
                    }
                }
            }
        }
    }

    pub fn load_frame(&self, frame: usize) -> Option<RetainedImage> {
        self.images.get(frame)
            .map(|path| {
                RetainedImage::from_image_bytes(
                    path.clone(),
                    &std::fs::read(path.clone()).unwrap(),
                ).unwrap()
            })
    }
}
