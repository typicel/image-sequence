use std::default::Default;
use std::path::{Path};
use std::sync::{Arc, Mutex};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use crate::image_sequence::ImageSequence;

pub struct ImageSequencerApp {
    watcher: RecommendedWatcher,
    image_sequence: Arc<Mutex<Option<ImageSequence>>>,
    idx: usize,
    playing: bool,
}

impl ImageSequencerApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let image_sequence = Arc::new(Mutex::new(Option::<ImageSequence>::None));

        let watcher_seq = image_sequence.clone();
        let watcher = notify::recommended_watcher(move |res| match res {
            Ok(notify::Event { kind, .. }) => {
                if kind.is_modify() || kind.is_create() || kind.is_remove() {
                    let mut image_sequence = watcher_seq.lock().unwrap();
                    if let Some(image_sequence) = &mut *image_sequence {
                        image_sequence.reload();
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }).unwrap();

        Self {
            image_sequence,
            watcher,
            idx: 0,
            playing: true,
        }
    }
}

impl eframe::App for ImageSequencerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            let mut image_sequence = self.image_sequence.lock().unwrap();

            if let Some(image_sequence) = &mut *image_sequence {
                ui.heading(format!("Current Directory: {:?}", image_sequence.directory));

                if !image_sequence.images.is_empty() {
                    if self.idx >= image_sequence.images.len() {
                        self.idx = image_sequence.images.len();
                    }

                    if let Some(img) = image_sequence.load_frame(self.idx) {
                        ui.add(egui::Image::new(img.texture_id(ctx), img.size_vec2()));
                        ctx.request_repaint();
                    }

                    ui.horizontal(|ui| {
                        ui.label(format!("{} / {}", self.idx + 1, image_sequence.images.len()));
                        ui.add(egui::Slider::new(&mut self.idx, 0..=image_sequence.images.len() - 1));
                    });

                    if self.playing {
                        self.idx = (self.idx + 1) % image_sequence.images.len();
                    }
                }
            }

            if ui.button("Open Directory").clicked() {
                let result = nfd::open_pick_folder(None).unwrap();

                match result {
                    nfd::Response::Okay(path) => {
                        println!("Path = {:?}", path);
                        let path = Path::new(path.as_str());

                        if let Some(image_sequence) = &mut *image_sequence {
                            self.watcher.unwatch(&image_sequence.directory).unwrap();
                        }

                        self.watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

                        let mut new_sequence = ImageSequence::new(&path);
                        new_sequence.reload();
                        *image_sequence = Some(new_sequence);
                    }

                    _ => {}
                }
            }

            if let Some(image_sequence) = &mut *image_sequence {
                if ui.button("Reload Directory").clicked() {
                    image_sequence.reload();
                }
            }

            // stops the animation but doesn't clear directory
            if ui.button(if self.playing { "Stop" } else { "Play" }).clicked() {
                self.playing = !self.playing;
            }
        });
    }
}
