use image::RgbaImage;

pub fn apply_crop(img_buffer: &mut RgbaImage, sx: u32, sy: u32, ex: u32, ey: u32) -> Option<RgbaImage>
{
    let min_x = sx.min(ex);
    let min_y = sy.min(ey);
    let width = sx.max(ex) - min_x;
    let height = sy.max(ey) - min_y;

    if width > 0 && height > 0
    {
        let sub_img = image::imageops::crop(img_buffer, min_x, min_y, width, height);
        Some(sub_img.to_image())
    }
    else
    {
        None
    }
}
