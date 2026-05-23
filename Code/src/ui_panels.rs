use eframe::egui;
use image::Rgba;
use crate::{QuasarApp, Tool};
use crate::theme;

/// Affiche la barre de menu supérieure (sauvegarder, ouvrir, infos fichier).
pub fn show_top_bar(app: &mut QuasarApp, ctx: &egui::Context)
{
    egui::TopBottomPanel::top("menu_bar")
        .frame(egui::Frame::new()
            .fill(theme::TOP_BAR_BG)
            .inner_margin(egui::Margin::symmetric(16, 8)))
        .show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            // App title
            ui.label(egui::RichText::new("Quasar").size(16.0).strong().color(theme::ACCENT));
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // ── Undo / Redo ──
            let btn_undo = egui::Button::new(
                egui::RichText::new("⬅").size(13.0)
            ).fill(theme::TOP_BTN_BG);
            
            let btn_redo = egui::Button::new(
                egui::RichText::new("➡").size(13.0)
            ).fill(theme::TOP_BTN_BG);

            if ui.add_enabled(!app.undo_history.is_empty(), btn_undo).on_hover_text("Annuler (Ctrl+Z)").clicked() {
                app.undo();
            }
            if ui.add_enabled(!app.redo_history.is_empty(), btn_redo).on_hover_text("Rétablir (Ctrl+Shift+Z)").clicked() {
                app.redo();
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // ── Sauvegarder ──
            let btn_save = egui::Button::new(
                egui::RichText::new("💾 Sauvegarder").size(13.0)
            ).fill(theme::TOP_BTN_BG);
            if ui.add(btn_save).clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Sauvegarder l'image sous")
                    .set_file_name(&app.image_name)
                    .add_filter("Image PNG", &["png"])
                    .add_filter("Image JPEG", &["jpg", "jpeg"])
                    .add_filter("Toutes les images", &["png", "jpg", "jpeg", "webp", "bmp"])
                    .save_file()
                {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    let result = if ext == "jpg" || ext == "jpeg"
                    {
                        let img_rgb = image::DynamicImage::ImageRgba8(app.image_buffer.clone()).into_rgb8();
                        let file = std::fs::File::create(&path);
                        match file {
                            Ok(mut f) => {
                                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut f, 100);
                                encoder.encode(img_rgb.as_raw(), img_rgb.width(), img_rgb.height(), image::ExtendedColorType::Rgb8.into())
                                    .map_err(|e| image::ImageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
                            },
                            Err(e) => Err(image::ImageError::IoError(e)),
                        }
                    }
                    else
                    {
                        app.image_buffer.save(&path)
                    };

                    match result
                    {
                        Ok(_) => {
                            log::info!("Image successfully saved to {:?}", path);
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                app.image_name = file_name.to_string();
                            }
                        },
                        Err(e) => log::error!("Failed to save image to {:?}: {}", path, e),
                    }
                }
            }

            // ── Ouvrir ──
            let btn_open = egui::Button::new(
                egui::RichText::new("📂 Ouvrir").size(13.0)
            ).fill(theme::TOP_BTN_BG);
            if ui.add(btn_open).clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Ouvrir une image")
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
                    .add_filter("Toutes les images", &["*"])
                    .pick_file()
                {
                    match image::open(&path)
                    {
                        Ok(img) =>
                        {
                            app.image_buffer = img.into_rgba8();
                            app.original_image = app.image_buffer.clone();
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                app.image_name = file_name.to_string();
                            }
                            app.zoom_level = 1.0;
                            app.texture = None;
                            app.pre_filter_image = None;
                            app.filter_opacity = 0.0;
                        }
                        Err(e) =>
                        {
                            log::error!("Failed to load {:?}: {}", path, e);
                        }
                    }
                }
            }

            // File name display on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(&app.image_name).size(12.0).color(theme::TEXT_MUTED));
                let dims = format!("{}×{}", app.image_buffer.width(), app.image_buffer.height());
                ui.label(egui::RichText::new(dims).size(11.0).color(theme::TEXT_DIM));
            });
        });
    });
}

/// Affiche la barre latérale gauche (outils, paramètres, couleurs).
pub fn show_sidebar(app: &mut QuasarApp, ctx: &egui::Context)
{
    egui::SidePanel::left("toolbar")
        .default_width(220.0)
        .frame(egui::Frame::new()
            .fill(theme::SIDEBAR_BG)
            .inner_margin(egui::Margin::symmetric(12, 12)))
        .show(ctx, |ui| {

        // ── Tools Section ──
        ui.add_space(4.0);
        ui.label(egui::RichText::new("🛠  OUTILS").size(12.0).strong().color(theme::ACCENT));
        ui.add_space(4.0);

        let tools = [
            (Tool::Pointer,   "👆", "Pointeur"),
            (Tool::Selection, "🎯", "Sélection"),
            (Tool::Pencil,    "🖊", "Crayon"),
            (Tool::Eraser,    "🧽", "Gomme"),
            (Tool::Rectangle, "🔲", "Rectangle"),
            (Tool::Triangle,  "🔺", "Triangle"),
            (Tool::Circle,    "⭕", "Cercle"),
            (Tool::Pipette,   "💧", "Pipette"),
            (Tool::Filter,    "🎨", "Filtre"),
            (Tool::Rognage,   "✂",  "Rogner"),
        ];

        // Tool grid: 2 columns
        egui::Grid::new("tool_grid")
            .num_columns(2)
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for (i, (tool, icon, label)) in tools.iter().enumerate() {
                    let is_active = app.active_tool == *tool;
                    let fill = if is_active { theme::BTN_ACTIVE_BG } else { theme::BTN_BG };
                    let text_color = if is_active { egui::Color32::WHITE } else { theme::TEXT_INACTIVE };

                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{} {}", icon, label))
                            .size(12.0)
                            .color(text_color)
                    )
                    .fill(fill)
                    .min_size(egui::vec2(88.0, 32.0));

                    if ui.add(btn).clicked() {
                        app.active_tool = *tool;
                        if app.active_tool != Tool::Filter {
                            if app.pre_filter_image.is_some() {
                                app.pre_filter_image = None;
                                app.filter_opacity = 0.0;
                            }
                        }
                    }

                    if i % 2 == 1 { ui.end_row(); }
                }
            });

        ui.add_space(16.0);

        // ── Separator ──
        ui.painter().line_segment(
            [ui.cursor().min, egui::pos2(ui.cursor().min.x + ui.available_width(), ui.cursor().min.y)],
            egui::Stroke::new(1.0, theme::SEPARATOR),
        );
        ui.add_space(8.0);

        // ── Parameters Section ──
        ui.label(egui::RichText::new("⚙  PARAMÈTRES").size(12.0).strong().color(theme::ACCENT));
        ui.add_space(6.0);

        egui::Frame::new()
            .fill(theme::FRAME_BG)
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Taille du trait").size(11.0).color(theme::TEXT_MUTED));
                ui.spacing_mut().slider_width = 120.0;
                ui.add(egui::Slider::new(&mut app.brush_size, 1..=50));
                
                if app.active_tool == Tool::Filter {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Opacité du filtre").size(11.0).color(theme::TEXT_MUTED));
                    
                    let mut opacity_val = (app.filter_opacity * 50.0).round() as i32;
                    if opacity_val == 0 { opacity_val = 1; }
                    let filter_slider = ui.add(egui::Slider::new(&mut opacity_val, 1..=50));
                    
                    if filter_slider.drag_started() {
                        if app.pre_filter_image.is_none() {
                            app.save_state();
                            app.pre_filter_image = Some(app.image_buffer.clone());
                        }
                    }
                    
                    if filter_slider.changed() {
                        app.filter_opacity = opacity_val as f32 / 50.0;
                        if app.pre_filter_image.is_none() {
                            app.save_state();
                            app.pre_filter_image = Some(app.image_buffer.clone());
                        }
                        
                        if let Some(ref base_img) = app.pre_filter_image {
                            app.image_buffer = base_img.clone();
                            if app.filter_opacity > 0.0 {
                                crate::filtres::apply_color_filter(&mut app.image_buffer, app.current_color, app.selection_mask, app.filter_opacity);
                            }
                            app.update_texture(ui.ctx()); // Force texture update without flickering
                        }
                    }
                }
                
                if app.selection_mask.is_some() {
                    ui.add_space(8.0);
                    if ui.button("❌ Annuler sélection").clicked() {
                        app.selection_mask = None;
                    }
                }
            });

        ui.add_space(16.0);

        // ── Separator ──
        ui.painter().line_segment(
            [ui.cursor().min, egui::pos2(ui.cursor().min.x + ui.available_width(), ui.cursor().min.y)],
            egui::Stroke::new(1.0, theme::SEPARATOR),
        );
        ui.add_space(8.0);

        // ── Colors Section ──
        ui.label(egui::RichText::new("🎨  COULEURS").size(12.0).strong().color(theme::ACCENT));
        ui.add_space(6.0);

        let palette = [
            ("Noir",    egui::Color32::BLACK),
            ("Blanc",   egui::Color32::WHITE),
            ("Rouge",   egui::Color32::from_rgb(220, 50, 50)),
            ("Vert",    egui::Color32::from_rgb(50, 200, 80)),
            ("Bleu",    egui::Color32::from_rgb(60, 120, 255)),
            ("Jaune",   egui::Color32::from_rgb(255, 220, 50)),
            ("Orange",  egui::Color32::from_rgb(255, 150, 30)),
            ("Violet",  egui::Color32::from_rgb(160, 60, 220)),
            ("Rose",    egui::Color32::from_rgb(255, 100, 150)),
            ("Cyan",    egui::Color32::from_rgb(50, 220, 220)),
            ("Marron",  egui::Color32::from_rgb(139, 90, 43)),
            ("Gris",    egui::Color32::from_rgb(128, 128, 128)),
        ];

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);
            for (_name, color) in palette
            {
                let is_active = app.current_color == Rgba([color.r(), color.g(), color.b(), 255]);
                let alloc_size = 28.0;
                let (alloc_rect, response) = ui.allocate_exact_size(egui::vec2(alloc_size, alloc_size), egui::Sense::click());

                if ui.is_rect_visible(alloc_rect)
                {
                    let inner_size = if is_active { 28.0 } else { 26.0 };
                    let rect = egui::Rect::from_center_size(alloc_rect.center(), egui::vec2(inner_size, inner_size));

                    let rounding = 6.0;
                    ui.painter().rect_filled(rect, rounding, color);
                    if is_active
                    {
                        ui.painter().rect_stroke(rect, rounding, egui::Stroke::new(2.5, egui::Color32::WHITE), egui::StrokeKind::Middle);
                        ui.painter().rect_stroke(rect.expand(2.0), rounding + 2.0, egui::Stroke::new(1.5, theme::ACCENT), egui::StrokeKind::Middle);
                    }
                    else if response.hovered()
                    {
                        ui.painter().rect_stroke(rect, rounding, egui::Stroke::new(1.5, egui::Color32::from_rgb(120, 120, 150)), egui::StrokeKind::Middle);
                    }
                    else
                    {
                        ui.painter().rect_stroke(rect, rounding, egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 75)), egui::StrokeKind::Middle);
                    }
                }

                if response.clicked()
                {
                    app.current_color = Rgba([color.r(), color.g(), color.b(), 255]);
                    
                    if app.pre_filter_image.is_some() {
                        app.pre_filter_image = None;
                        app.filter_opacity = 0.0;
                    }
                }

                if response.hovered() {
                    response.on_hover_text(_name);
                }
            }
        });

    });
}
