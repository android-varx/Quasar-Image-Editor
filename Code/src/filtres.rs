use image::{Rgba, RgbaImage};

pub fn apply_color_filter(img: &mut RgbaImage, filter_color: Rgba<u8>, mask: Option<(u32, u32, u32, u32)>, opacity: f32)
{
    let fr = filter_color[0] as f32 / 255.0;
    let fg = filter_color[1] as f32 / 255.0;
    let fb = filter_color[2] as f32 / 255.0;

    let width = img.width();

    img.pixels_mut().enumerate().for_each(|(i, pixel)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;

        if let Some((mx0, my0, mx1, my1)) = mask {
            if x < mx0 || x > mx1 || y < my0 || y > my1 {
                return;
            }
        }

        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let blend_amount = opacity;
        
        let new_r = r * (1.0 - blend_amount) + fr * blend_amount;
        let new_g = g * (1.0 - blend_amount) + fg * blend_amount;
        let new_b = b * (1.0 - blend_amount) + fb * blend_amount;

        pixel[0] = (new_r * 255.0).clamp(0.0, 255.0) as u8;
        pixel[1] = (new_g * 255.0).clamp(0.0, 255.0) as u8;
        pixel[2] = (new_b * 255.0).clamp(0.0, 255.0) as u8;
    });
}
