use std::path::PathBuf;
pub mod pipeline;
use pipeline::process_image;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::{GenericImageView, ImageFormat, RgbaImage};

struct LoadedImage {
    path: PathBuf,
    original: RgbaImage,
    displayed: RgbaImage,
    texture: Option<TextureHandle>,
    borders: Vec<Vec<(u32, u32)>>,
    current_obj: usize,
}

struct ImageBrowser {
    images: Vec<LoadedImage>,
    selected: usize,
    status: String,
}

impl ImageBrowser {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            images: Vec::new(),
            selected: 0,
            status: "Nenhuma imagem carregada".to_owned(),
        }
    }

    fn current(&self) -> Option<&LoadedImage> {
        self.images.get(self.selected)
    }

    fn current_mut(&mut self) -> Option<&mut LoadedImage> {
        self.images.get_mut(self.selected)
    }

    fn load_images(&mut self, paths: Vec<PathBuf>) {
        let mut loaded = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            match image::open(&path) {
                Ok(image) => {
                    let borders = process_image(&image);
                    loaded.push(LoadedImage {
                        path,
                        original: image.to_rgba8(),
                        displayed: image.to_rgba8(),
                        texture: None,
                        borders,
                        current_obj: 0,
                    });
                }
                Err(error) => errors.push(format!("{}: {error}", path.display())),
            }
        }

        self.images = loaded;
        self.selected = 0;
        self.status = if errors.is_empty() {
            format!("{} imagem(ns) carregada(s)", self.images.len())
        } else {
            format!(
                "{} imagem(ns) carregada(s); falha ao abrir: {}",
                self.images.len(),
                errors.join("; ")
            )
        };
    }

    fn reset_current(&mut self) {
        if let Some(image) = self.current_mut() {
            image.displayed = image.original.clone();
            image.texture = None;
            self.status = "Imagem restaurada".to_owned();
        }
    }

    fn texture_for_current(&mut self, ctx: &egui::Context) -> Option<&TextureHandle> {
        let image = self.current_mut()?;

        if image.texture.is_none() {
            let rgba = &image.displayed;
            let size = [rgba.width() as usize, rgba.height() as usize];
            let color_image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            image.texture = Some(ctx.load_texture(
                image.path.to_string_lossy(),
                color_image,
                TextureOptions::LINEAR,
            ));
        }

        image.texture.as_ref()
    }

    fn save_current(&mut self) {
        let Some(image) = self.current() else {
            return;
        };

        let Some(path) = rfd::FileDialog::new()
            .set_file_name("imagem-processada.png")
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .save_file()
        else {
            return;
        };

        let format = match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("jpg" | "jpeg") => ImageFormat::Jpeg,
            _ => ImageFormat::Png,
        };

        match image.displayed.save_with_format(&path, format) {
            Ok(()) => self.status = format!("Imagem salva em {}", path.display()),
            Err(error) => self.status = format!("Erro ao salvar imagem: {error}"),
        }
    }
}

impl eframe::App for ImageBrowser {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Carregar imagens").clicked()
                    && let Some(paths) = rfd::FileDialog::new()
                        .add_filter("Imagens", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
                        .pick_files()
                {
                    self.load_images(paths);
                }

                let has_image = !self.images.is_empty();
                ui.add_enabled_ui(has_image, |ui| {
                    if ui.button("Anterior").clicked() {
                        self.selected = self.selected.saturating_sub(1);
                        self.images[self.selected].current_obj = 0;
                    }
                    if ui.button("Próxima").clicked() {
                        self.selected = (self.selected + 1).min(self.images.len() - 1);
                        self.images[self.selected].current_obj = 0;
                    }

                    if ui.button("Alternar Objeto").clicked() {
                        let (_, selected_images) = self.images.split_at_mut(self.selected);
                        let image = &mut selected_images[0];

                        if !image.borders.is_empty() {
                            let new_obj = (image.current_obj + 1) % image.borders.len();
                            image.current_obj = new_obj;
                            self.reset_current();
                        }
                    }
                    if ui.button("Restaurar").clicked() {
                        self.reset_current();
                    }
                    if ui.button("Salvar como...").clicked() {
                        self.save_current();
                    }
                });
            });
        });

        egui::SidePanel::left("image_list")
            .resizable(true)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Imagens");
                ui.separator();

                for (index, image) in self.images.iter().enumerate() {
                    let name = image
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("imagem");
                    if ui
                        .selectable_label(self.selected == index, format!("{index}: {name}"))
                        .clicked()
                    {
                        self.selected = index;
                    }
                }
            });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(current) = self.current() else {
                ui.centered_and_justified(|ui| {
                    ui.label("Clique em “Carregar imagens” para começar.");
                });
                return;
            };

            let name = current
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("imagem");
            let dimensions = current.displayed.dimensions();
            ui.heading(format!(
                "{} ({}/{})",
                name,
                self.selected + 1,
                self.images.len()
            ));
            ui.label(format!("{} × {} px", dimensions.0, dimensions.1));
            ui.separator();

            

            let (_, selected_images) = self.images.split_at_mut(self.selected);
            let image = &mut selected_images[0];

            if !image.borders.is_empty() {
                let borders = &image.borders[image.current_obj];
                for (x, y) in borders {
                    image
                        .displayed
                        .put_pixel(*x, *y, image::Rgba([255, 0, 0, 255]));
                }
            }

            if let Some(texture) = self.texture_for_current(ctx) {
                let available = ui.available_size();
                let image_size = texture.size_vec2();
                let scale = (available.x / image_size.x)
                    .min(available.y / image_size.y)
                    .min(1.0);
                ui.add(egui::Image::from_texture(texture).fit_to_exact_size(image_size * scale));
            }
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Navegador de imagens",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(ImageBrowser::new(cc)))),
    )
}
