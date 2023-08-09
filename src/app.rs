use crate::image_sequence::ImageSequence;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rfd::FileDialog;
use std::default::Default;
use std::sync::{Arc, Mutex};

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
        })
        .unwrap();

        Self {
            image_sequence,
            watcher,
            idx: 0,
            playing: true,
        }
    }

    // pub fn open_directory(&mut self) {
    //     let mut image_sequence = self.image_sequence.lock().unwrap();

    //     let result = FileDialog::new().pick_folder();

    //     match result {
    //         Some(path) => {
    //             if let Some(image_sequence) = &mut *image_sequence {
    //                 self.watcher.unwatch(&image_sequence.directory).unwrap();
    //             }

    //             self.watcher
    //                 .watch(&path, RecursiveMode::NonRecursive)
    //                 .unwrap();

    //             let mut new_sequence = ImageSequence::new(&path);
    //             new_sequence.reload();
    //             *image_sequence = Some(new_sequence);
    //         }
    //         _ => {}
    //     }
    // }
}

impl eframe::App for ImageSequencerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let self_rc = Rc::new(RefCell::new(self));
        let mut image_sequence = self.image_sequence.lock().unwrap();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }

                    if ui.button("Open").clicked() {
                        // self.open_directory();
                        let result = FileDialog::new().pick_folder();

                        match result {
                            Some(path) => {
                                if let Some(image_sequence) = &mut *image_sequence {
                                    self.watcher.unwatch(&image_sequence.directory).unwrap();
                                }

                                self.watcher
                                    .watch(&path, RecursiveMode::NonRecursive)
                                    .unwrap();

                                let mut new_sequence = ImageSequence::new(&path);
                                new_sequence.reload();
                                *image_sequence = Some(new_sequence);
                            }
                            _ => {}
                        }
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                if ui
                    .button(if self.playing { "Stop" } else { "Play" })
                    .clicked()
                {
                    self.playing = !self.playing;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                if let Some(image_sequence) = &mut *image_sequence {
                    if self.idx >= image_sequence.images.len() {
                        self.idx = image_sequence.images.len();
                    }

                    if let Some(img) = image_sequence.load_frame(self.idx) {
                        // get the current visible space of the current widget that's visible
                        let avail_space = ui.available_size();

                        // size is max of available space or image size
                        let new_size = egui::Vec2::new(
                            avail_space.x.min(img.width() as f32),
                            avail_space.y.min(img.height() as f32),
                        );

                        ui.add(egui::Image::new(img.texture_id(ctx), new_size));

                        ctx.request_repaint();
                    }

                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} / {}",
                            self.idx + 1,
                            image_sequence.images.len()
                        ));
                        ui.add(egui::Slider::new(
                            &mut self.idx,
                            0..=image_sequence.images.len() - 1,
                        ));
                    });

                    if self.playing {
                        self.idx = (self.idx + 1) % image_sequence.images.len();
                    }
                } else {
                    ui.label("No image sequence loaded");
                }
            });
        });
    }
}
