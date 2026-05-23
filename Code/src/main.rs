mod crayon;
mod formes;
mod filtres;
mod rognage;
mod theme;
mod ui_panels;
mod canvas;

use eframe::egui;
use image::{Rgba, RgbaImage};

#[derive(PartialEq, Clone, Copy)]
pub enum Tool
{
    Pointer,
    Pencil,
    Eraser,
    Rectangle,
    Triangle,
    Circle,
    Pipette,
    Filter,
    Rognage,
    Selection,
}

pub struct QuasarApp
{
    pub original_image: RgbaImage,
    pub image_buffer: RgbaImage,
    pub texture: Option<egui::TextureHandle>,
    pub last_cursor_pos: Option<(u32, u32)>,
    pub active_tool: Tool,
    pub drag_start_pos: Option<egui::Pos2>,
    pub current_color: Rgba<u8>,
    pub brush_size: u32,
    pub image_name: String,
    pub zoom_level: f32,
    pub selection_mask: Option<(u32, u32, u32, u32)>,
    pub filter_opacity: f32,
    pub pre_filter_image: Option<RgbaImage>,
    pub undo_history: Vec<(RgbaImage, RgbaImage, Option<(u32, u32, u32, u32)>)>,
    pub redo_history: Vec<(RgbaImage, RgbaImage, Option<(u32, u32, u32, u32)>)>,
}

impl QuasarApp
{
    fn new(_cc: &eframe::CreationContext<'_>, image_name: String) -> Self
    {
        let fallback_path = format!("../Images/{}", image_name);
        
        let image_path = if std::path::Path::new(&image_name).exists()
        {
            &image_name
        }
        else
        {
            &fallback_path
        };

        let image_buffer = match image::open(image_path)
        {
            Ok(img) =>
            {
                log::info!("Image loaded successfully from {}", image_path);
                img.into_rgba8()
            }
            Err(err) =>
            {
                log::error!("Failed to load image {}: {}", image_path, err);
                let mut img = RgbaImage::new(800, 600);
                for pixel in img.pixels_mut()
                {
                    *pixel = Rgba([255, 255, 255, 255]);
                }
                img
            }
        };

        Self
        {
            original_image: image_buffer.clone(),
            image_buffer,
            texture: None,
            last_cursor_pos: None,
            active_tool: Tool::Pencil,
            drag_start_pos: None,
            current_color: Rgba([0, 0, 0, 255]),
            brush_size: 1,
            image_name,
            zoom_level: 1.0,
            selection_mask: None,
            filter_opacity: 0.0,
            pre_filter_image: None,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        }
    }

    pub fn update_texture(&mut self, ctx: &egui::Context)
    {
        let size = [
            self.image_buffer.width() as _,
            self.image_buffer.height() as _,
        ];
        let pixels = self.image_buffer.as_flat_samples();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        if let Some(texture) = &mut self.texture
        {
            texture.set(color_image, egui::TextureOptions::LINEAR);
        }
        else
        {
            self.texture = Some(ctx.load_texture(
                "main_image",
                color_image,
                egui::TextureOptions::LINEAR,
            ));
        }
    }

    pub fn save_state(&mut self)
    {
        if self.undo_history.len() > 30 {
            self.undo_history.remove(0);
        }
        self.undo_history.push((self.image_buffer.clone(), self.original_image.clone(), self.selection_mask));
        self.redo_history.clear();
    }

    pub fn undo(&mut self)
    {
        if let Some(prev_state) = self.undo_history.pop() {
            self.redo_history.push((self.image_buffer.clone(), self.original_image.clone(), self.selection_mask));
            self.image_buffer = prev_state.0;
            self.original_image = prev_state.1;
            self.selection_mask = prev_state.2;
            self.texture = None;
            self.pre_filter_image = None;
            self.filter_opacity = 0.0;
        }
    }

    pub fn redo(&mut self)
    {
        if let Some(next_state) = self.redo_history.pop() {
            self.undo_history.push((self.image_buffer.clone(), self.original_image.clone(), self.selection_mask));
            self.image_buffer = next_state.0;
            self.original_image = next_state.1;
            self.selection_mask = next_state.2;
            self.texture = None;
            self.pre_filter_image = None;
            self.filter_opacity = 0.0;
        }
    }
}

impl eframe::App for QuasarApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        theme::apply_theme(ctx);

        if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
            self.redo();
        } else if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.undo();
        }

        if self.texture.is_none()
        {
            self.update_texture(ctx);
        }

        // ── Barre supérieure (Sauvegarder, Ouvrir) ──
        ui_panels::show_top_bar(self, ctx);

        // ── Barre latérale (Outils, Paramètres, Couleurs) ──
        ui_panels::show_sidebar(self, ctx);

        // ── Panneau central (Canvas de dessin) ──
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture
            {
                let texture_id = texture.id();
                let scroll_area = egui::ScrollArea::both()
                    .auto_shrink([false, false]);

                scroll_area.show(ui, |ui| {
                    let image_size = egui::vec2(
                        self.image_buffer.width() as f32 * self.zoom_level,
                        self.image_buffer.height() as f32 * self.zoom_level,
                    );

                    let mut img = egui::Image::new((texture_id, image_size)).fit_to_exact_size(image_size);
                    if self.active_tool == Tool::Pointer {
                        img = img.sense(egui::Sense::drag());
                    } else {
                        img = img.sense(egui::Sense::click_and_drag());
                    }

                    let response = ui.add(img);

                    // Panning
                    if self.active_tool == Tool::Pointer && response.dragged() {
                        let delta = response.drag_delta();
                        ui.scroll_with_delta(delta);
                    }

                    // Zoom molette
                    if response.hovered() {
                        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                        if scroll_delta != 0.0 {
                            let zoom_speed = 0.001;
                            let new_zoom = self.zoom_level * (1.0 + scroll_delta * zoom_speed);
                            self.zoom_level = new_zoom.clamp(0.01, 8.0);
                        }
                    }

                    let painter = ui.painter();
                    let rect = response.rect;
                    let scale_x = 1.0 / self.zoom_level;
                    let scale_y = 1.0 / self.zoom_level;

                    let draw_color_pixels = self.current_color;
                    let draw_color_egui = egui::Color32::from_rgba_unmultiplied(
                        self.current_color[0],
                        self.current_color[1],
                        self.current_color[2],
                        self.current_color[3],
                    );

                    if let Some((sx, sy, ex, ey)) = self.selection_mask {
                        let rect_sel = egui::Rect::from_two_pos(
                            rect.min + egui::vec2(sx as f32 / scale_x, sy as f32 / scale_y),
                            rect.min + egui::vec2(ex as f32 / scale_x, ey as f32 / scale_y),
                        );
                        painter.rect_stroke(rect_sel, 0.0, (1.0, egui::Color32::from_rgb(100, 200, 255)), egui::StrokeKind::Middle);
                        // Make it slightly transparent inside
                        painter.rect_filled(rect_sel, 0.0, egui::Color32::from_rgba_unmultiplied(100, 200, 255, 30));
                    }

                    if self.active_tool != Tool::Pointer {
                        if response.drag_started() || (response.clicked() && (self.active_tool == Tool::Pencil || self.active_tool == Tool::Eraser))
                        {
                            if self.pre_filter_image.is_some() {
                                self.pre_filter_image = None;
                                self.filter_opacity = 0.0;
                            }
                        }

                        if response.drag_started()
                        {
                            self.save_state();
                            self.drag_start_pos = response.interact_pointer_pos();
                            self.last_cursor_pos = None;
                        }

                        if response.dragged() || (response.clicked() && (self.active_tool == Tool::Pencil || self.active_tool == Tool::Eraser))
                        {
                            if response.clicked() {
                                self.save_state();
                            }

                            if let Some(pos) = response.interact_pointer_pos()
                            {
                                let local_pos = pos - rect.min;
                                let img_x = (local_pos.x * scale_x) as u32;
                                let img_y = (local_pos.y * scale_y) as u32;

                                match self.active_tool
                                {
                                    Tool::Pencil =>
                                    {
                                        canvas::draw_pencil(self, img_x, img_y, draw_color_pixels);
                                    }
                                    Tool::Eraser =>
                                    {
                                        canvas::erase_pencil(self, img_x, img_y);
                                    }
                                    Tool::Rectangle | Tool::Triangle | Tool::Circle | Tool::Rognage | Tool::Selection =>
                                    {
                                        if let Some(start_pos) = self.drag_start_pos
                                        {
                                            if self.active_tool == Tool::Rectangle || self.active_tool == Tool::Selection
                                            {
                                                let rect_preview = egui::Rect::from_two_pos(start_pos, pos);
                                                let is_selection = self.active_tool == Tool::Selection;
                                                let color = if is_selection { egui::Color32::GRAY } else { draw_color_egui };
                                                let thickness = if is_selection { 1.0 } else { self.brush_size as f32 };
                                                painter.rect_stroke(rect_preview, 0.0, (thickness, color), egui::StrokeKind::Middle);
                                            }
                                            else if self.active_tool == Tool::Rognage
                                            {
                                                let rect_preview = egui::Rect::from_two_pos(start_pos, pos);
                                                let img_rect = egui::Rect::from_min_size(rect.min, egui::vec2(self.image_buffer.width() as f32 * scale_x.recip(), self.image_buffer.height() as f32 * scale_y.recip()));
                                                
                                                // Grise l'extérieur du crop
                                                let dark_overlay = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150);
                                                painter.rect_filled(egui::Rect::from_two_pos(img_rect.min, egui::pos2(img_rect.max.x, rect_preview.min.y)), 0.0, dark_overlay); // top
                                                painter.rect_filled(egui::Rect::from_two_pos(egui::pos2(img_rect.min.x, rect_preview.max.y), img_rect.max), 0.0, dark_overlay); // bottom
                                                painter.rect_filled(egui::Rect::from_two_pos(egui::pos2(img_rect.min.x, rect_preview.min.y), egui::pos2(rect_preview.min.x, rect_preview.max.y)), 0.0, dark_overlay); // left
                                                painter.rect_filled(egui::Rect::from_two_pos(egui::pos2(rect_preview.max.x, rect_preview.min.y), egui::pos2(img_rect.max.x, rect_preview.max.y)), 0.0, dark_overlay); // right

                                                painter.rect_stroke(rect_preview, 0.0, (2.0, egui::Color32::WHITE), egui::StrokeKind::Middle);
                                                painter.rect_stroke(rect_preview.expand(1.0), 0.0, (1.0, egui::Color32::BLACK), egui::StrokeKind::Middle);
                                            }
                                            else if self.active_tool == Tool::Circle
                                            {
                                                let radius = start_pos.distance(pos);
                                                painter.circle_stroke(start_pos, radius, (self.brush_size as f32, draw_color_egui));
                                            }
                                            else // Triangle
                                            {
                                                let p1 = egui::pos2((start_pos.x + pos.x) / 2.0, pos.y);
                                                let p2 = egui::pos2(start_pos.x, start_pos.y);
                                                let p3 = egui::pos2(pos.x, start_pos.y);

                                                let thick = self.brush_size as f32;
                                                painter.line_segment([p1, p2], (thick, draw_color_egui));
                                                painter.line_segment([p2, p3], (thick, draw_color_egui));
                                                painter.line_segment([p3, p1], (thick, draw_color_egui));
                                            }
                                        }
                                    }
                                    Tool::Pipette =>
                                    {
                                        if img_x < self.image_buffer.width() && img_y < self.image_buffer.height()
                                        {
                                            self.current_color = *self.image_buffer.get_pixel(img_x, img_y);
                                        }
                                    }
                                    Tool::Filter =>
                                    {
                                        // Le filtre est maintenant appliqué en temps réel via le slider dans ui_panels.rs
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if response.drag_stopped()
                        {
                            if let (Some(start_pos), Some(end_pos)) = (self.drag_start_pos, response.interact_pointer_pos())
                            {
                                if self.active_tool == Tool::Rectangle || self.active_tool == Tool::Triangle || self.active_tool == Tool::Circle || self.active_tool == Tool::Rognage || self.active_tool == Tool::Selection
                                {
                                    let local_start = start_pos - rect.min;
                                    let local_end = end_pos - rect.min;

                                    let start_x = (local_start.x * scale_x) as u32;
                                    let start_y = (local_start.y * scale_y) as u32;
                                    let end_x = (local_end.x * scale_x) as u32;
                                    let end_y = (local_end.y * scale_y) as u32;

                                    let w = self.image_buffer.width().saturating_sub(1);
                                    let h = self.image_buffer.height().saturating_sub(1);

                                    let sx = start_x.min(w);
                                    let sy = start_y.min(h);
                                    let ex = end_x.min(w);
                                    let ey = end_y.min(h);

                                    match self.active_tool
                                    {
                                        Tool::Rectangle =>
                                        {
                                            formes::draw_rect_pixels(&mut self.image_buffer, sx, sy, ex, ey, draw_color_pixels, self.brush_size, self.selection_mask);
                                            self.update_texture(ctx);
                                        }
                                        Tool::Triangle =>
                                        {
                                            formes::draw_triangle_pixels(&mut self.image_buffer, sx, sy, ex, ey, draw_color_pixels, self.brush_size, self.selection_mask);
                                            self.update_texture(ctx);
                                        }
                                        Tool::Circle =>
                                        {
                                            let radius = ((ex as f32 - start_x as f32).powi(2) + (ey as f32 - start_y as f32).powi(2)).sqrt() as i32;
                                            formes::draw_circle_pixels(&mut self.image_buffer, start_x as i32, start_y as i32, radius, draw_color_pixels, self.brush_size, self.selection_mask);
                                            self.update_texture(ctx);
                                        }
                                        Tool::Rognage =>
                                        {
                                            if let Some(new_img) = rognage::apply_crop(&mut self.image_buffer, sx, sy, ex, ey)
                                            {
                                                self.image_buffer = new_img;
                                                self.original_image = self.image_buffer.clone();
                                                self.texture = None;
                                                self.selection_mask = None;
                                            }
                                        }
                                        Tool::Selection =>
                                        {
                                            if sx == ex && sy == ey {
                                                self.selection_mask = None;
                                            } else {
                                                self.selection_mask = Some((sx.min(ex), sy.min(ey), sx.max(ex), sy.max(ey)));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            self.drag_start_pos = None;
                        }
                    }
                }); // fin scroll_area

                // ── Panneau flottant zoom ──
                let mut zoom_percent = (self.zoom_level * 100.0).round() as u32;
                egui::Area::new("zoom_area".into())
                    .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
                    .show(ctx, |ui| {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_premultiplied(25, 25, 35, 220))
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::symmetric(12, 8))
                            .stroke(egui::Stroke::new(1.0, theme::FLOAT_BORDER))
                            .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔍").size(14.0));
                                if ui.add(egui::Slider::new(&mut zoom_percent, 1..=800).text("%")).changed() {
                                    self.zoom_level = zoom_percent as f32 / 100.0;
                                }
                            });
                        });
                    });
            }
        });
    }
}

fn main() -> eframe::Result<()>
{
    env_logger::init();
    
    let mut image_name = std::env::args().nth(1).unwrap_or_else(|| "img1.png".to_string());
    
    if !image_name.ends_with(".png") && !image_name.ends_with(".jpg") && !image_name.ends_with(".jpeg")
    {
        image_name.push_str(".png");
    }
    
    let options = eframe::NativeOptions
    {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Quasar - Editeur d'image"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Quasar",
        options,
        Box::new(|cc| Ok(Box::new(QuasarApp::new(cc, image_name)))),
    )
}
