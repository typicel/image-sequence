use egui_extras::RetainedImage;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use walkdir::WalkDir;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ImageSequencerApp {
    watcher: Option<RecommendedWatcher>,
    directory: Option<String>,
    images: Vec<String>,
    idx: usize,
}

impl Default for ImageSequencerApp {
    fn default() -> Self {
        Self {
            watcher: None,
            directory: None,
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
    fn watch_directory(&mut self, directory: &str) {
        let self_rc = Rc::new(RefCell::new(self.clone()));
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::event::Event, notify::Error>| match res {
            Ok(event) => match event.kind {
                notify::EventKind::Modify(_) => {
                    let mut self_ref = self_rc.borrow_mut();
                    self.load_images_from_directory(directory);
                }
                _ => {}
            },
            Err(e) => println!("watch error: {:?}", e),
        })
        .unwrap();

        watcher
            .watch(Path::new(directory), RecursiveMode::Recursive)
            .unwrap();

        self.watcher = Some(watcher);
    }

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
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.directory.is_some() {
                ui.heading(format!(
                    "Current Directory: {}",
                    self.directory.as_ref().unwrap()
                ));

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
                        self.watch_directory(path.as_str());
                        self.load_images_from_directory(path.as_str());
                        self.directory = Some(path);
                    }
                    nfd::Response::OkayMultiple(paths) => {
                        println!("Paths = {:?}", paths);
                    }
                    nfd::Response::Cancel => println!("User canceled"),
                }
            }

            if !self.images.is_empty() {
                if ui.button("Reload Directory").clicked() {}
            }

            // stops the animation but doesn't clear directory
            if ui.button("Stop").clicked() {
                self.images.clear();
            }
        });
    }
}
