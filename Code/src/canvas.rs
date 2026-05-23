use eframe::egui;
use image::Rgba;
use crate::QuasarApp;

/// Envoie uniquement la zone modifiée au GPU (mise à jour partielle de texture).
pub fn partial_texture_update(app: &mut QuasarApp, min_x: u32, min_y: u32, max_x: u32, max_y: u32)
{
    if min_x > max_x || min_y > max_y { return; }

    let rect_w = max_x - min_x + 1;
    let rect_h = max_y - min_y + 1;
    let mut partial_pixels = Vec::with_capacity((rect_w * rect_h * 4) as usize);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = app.image_buffer.get_pixel(x, y);
            partial_pixels.extend_from_slice(&p.0);
        }
    }

    let partial_image = egui::ColorImage::from_rgba_unmultiplied(
        [rect_w as usize, rect_h as usize],
        &partial_pixels
    );

    if let Some(tex) = &mut app.texture {
        tex.set_partial([min_x as usize, min_y as usize], partial_image, egui::TextureOptions::LINEAR);
    }
}

fn is_in_mask(x: u32, y: u32, mask: Option<(u32, u32, u32, u32)>) -> bool {
    if let Some((mx0, my0, mx1, my1)) = mask {
        if x < mx0 || x > mx1 || y < my0 || y > my1 {
            return false;
        }
    }
    true
}

/// Dessine avec l'outil Crayon et met à jour partiellement la texture.
pub fn draw_pencil(app: &mut QuasarApp, img_x: u32, img_y: u32, color: Rgba<u8>)
{
    if img_x >= app.image_buffer.width() || img_y >= app.image_buffer.height() {
        return;
    }

    if let Some((last_x, last_y)) = app.last_cursor_pos
    {
        crate::crayon::draw_line(&mut app.image_buffer, last_x, last_y, img_x, img_y, color, app.brush_size, app.selection_mask);

        let min_x = img_x.min(last_x).saturating_sub(app.brush_size);
        let max_x = img_x.max(last_x).saturating_add(app.brush_size).min(app.image_buffer.width() - 1);
        let min_y = img_y.min(last_y).saturating_sub(app.brush_size);
        let max_y = img_y.max(last_y).saturating_add(app.brush_size).min(app.image_buffer.height() - 1);

        partial_texture_update(app, min_x, min_y, max_x, max_y);
    }
    else
    {
        if app.brush_size <= 1
        {
            if is_in_mask(img_x, img_y, app.selection_mask) {
                app.image_buffer.put_pixel(img_x, img_y, color);
            }
        }
        else
        {
            crate::crayon::draw_filled_circle(&mut app.image_buffer, img_x as i32, img_y as i32, ((app.brush_size as i32) - 1) / 2, color, app.selection_mask);
        }

        let min_x = img_x.saturating_sub(app.brush_size);
        let max_x = img_x.saturating_add(app.brush_size).min(app.image_buffer.width() - 1);
        let min_y = img_y.saturating_sub(app.brush_size);
        let max_y = img_y.saturating_add(app.brush_size).min(app.image_buffer.height() - 1);

        partial_texture_update(app, min_x, min_y, max_x, max_y);
    }

    app.last_cursor_pos = Some((img_x, img_y));
}

/// Gomme qui restaure l'image d'origine.
pub fn erase_pencil(app: &mut QuasarApp, img_x: u32, img_y: u32)
{
    if img_x >= app.image_buffer.width() || img_y >= app.image_buffer.height() {
        return;
    }

    if let Some((last_x, last_y)) = app.last_cursor_pos
    {
        crate::crayon::erase_line(&mut app.image_buffer, &app.original_image, last_x, last_y, img_x, img_y, app.brush_size, app.selection_mask);

        let min_x = img_x.min(last_x).saturating_sub(app.brush_size);
        let max_x = img_x.max(last_x).saturating_add(app.brush_size).min(app.image_buffer.width() - 1);
        let min_y = img_y.min(last_y).saturating_sub(app.brush_size);
        let max_y = img_y.max(last_y).saturating_add(app.brush_size).min(app.image_buffer.height() - 1);

        partial_texture_update(app, min_x, min_y, max_x, max_y);
    }
    else
    {
        if app.brush_size <= 1
        {
            if img_x < app.original_image.width() && img_y < app.original_image.height()
            {
                if is_in_mask(img_x, img_y, app.selection_mask) {
                    let orig_color = *app.original_image.get_pixel(img_x, img_y);
                    app.image_buffer.put_pixel(img_x, img_y, orig_color);
                }
            }
        }
        else
        {
            crate::crayon::erase_filled_circle(&mut app.image_buffer, &app.original_image, img_x as i32, img_y as i32, ((app.brush_size as i32) - 1) / 2, app.selection_mask);
        }

        let min_x = img_x.saturating_sub(app.brush_size);
        let max_x = img_x.saturating_add(app.brush_size).min(app.image_buffer.width() - 1);
        let min_y = img_y.saturating_sub(app.brush_size);
        let max_y = img_y.saturating_add(app.brush_size).min(app.image_buffer.height() - 1);

        partial_texture_update(app, min_x, min_y, max_x, max_y);
    }

    app.last_cursor_pos = Some((img_x, img_y));
}
