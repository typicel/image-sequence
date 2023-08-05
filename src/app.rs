use egui_extras::RetainedImage;
use walkdir::WalkDir;
pub struct ImageSequencerApp {
    images: Vec<String>,
    idx: usize,
}

impl Default for ImageSequencerApp {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            idx: 0,
        }
    }
}

impl ImageSequencerApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl ImageSequencerApp {
    fn load_images_from_directory(&mut self, directory: &str) {
        self.images.clear();

        let mut entries: Vec<_> = WalkDir::new(directory)
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

    fn load_current_image(&self) -> Option<egui_extras::RetainedImage> {
        Some(
            RetainedImage::from_image_bytes(
                self.images[self.idx].clone(),
                &std::fs::read(self.images[self.idx].clone()).unwrap(),
            )
            .unwrap(),
        )
    }
}

impl eframe::App for ImageSequencerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.images.is_empty() {
                if let Some(img) = self.load_current_image() {
                    ui.add(egui::Image::new(img.texture_id(ctx), img.size_vec2()));

                    self.idx = (self.idx + 1) % self.images.len();
                    ctx.request_repaint();
                }

                ui.horizontal(|ui| {
                    ui.label(format!("{} / {}", self.idx + 1, self.images.len()));
                    ui.add(egui::Slider::new(&mut self.idx, 0..=self.images.len() - 1));
                });
            }

            if ui.button("Open Directory").clicked() {
                let result = nfd::open_pick_folder(None).unwrap_or_else(|e| {
                    panic!("{}", e);
                });

                match result {
                    nfd::Response::Okay(path) => {
                        println!("Path = {:?}", path);
                        self.load_images_from_directory(path.as_str());
                    }
                    nfd::Response::OkayMultiple(paths) => {
                        println!("Paths = {:?}", paths);
                    }
                    nfd::Response::Cancel => println!("User canceled"),
                }
            }

            if ui.button("Stop").clicked() {
                self.images.clear();
            }
        });
    }
}
