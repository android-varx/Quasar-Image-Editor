use image::{Rgba, RgbaImage};
use crate::crayon::{draw_line, draw_filled_circle};

fn put_pixel_masked(img: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>, mask: Option<(u32, u32, u32, u32)>) {
    if let Some((mx0, my0, mx1, my1)) = mask {
        if x < mx0 || x > mx1 || y < my0 || y > my1 {
            return;
        }
    }
    img.put_pixel(x, y, color);
}

pub fn draw_rect_pixels(img: &mut RgbaImage, mut x0: u32, mut y0: u32, mut x1: u32, mut y1: u32, color: Rgba<u8>, thickness: u32, mask: Option<(u32, u32, u32, u32)>)
{
    if x0 > x1
    {
        std::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y1
    {
        std::mem::swap(&mut y0, &mut y1);
    }
    
    let t = thickness.saturating_sub(1);

    for y_offset in 0..=t
    {
        for x in x0..=x1
        {
            if x < img.width()
            {
                if y0 + y_offset < img.height()
                {
                    put_pixel_masked(img, x, y0 + y_offset, color, mask);
                }
                if y1 >= y_offset && y1 - y_offset < img.height()
                {
                    put_pixel_masked(img, x, y1 - y_offset, color, mask);
                }
            }
        }
    }
    
    for x_offset in 0..=t
    {
        for y in y0..=y1
        {
            if y < img.height()
            {
                if x0 + x_offset < img.width()
                {
                    put_pixel_masked(img, x0 + x_offset, y, color, mask);
                }
                if x1 >= x_offset && x1 - x_offset < img.width()
                {
                    put_pixel_masked(img, x1 - x_offset, y, color, mask);
                }
            }
        }
    }
}

pub fn draw_triangle_pixels(img: &mut RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgba<u8>, thickness: u32, mask: Option<(u32, u32, u32, u32)>)
{
    let pt1 = ((x0 + x1) / 2, y1);
    let pt2 = (x0, y0);
    let pt3 = (x1, y0);

    draw_line(img, pt1.0, pt1.1, pt2.0, pt2.1, color, thickness, mask);
    draw_line(img, pt2.0, pt2.1, pt3.0, pt3.1, color, thickness, mask);
    draw_line(img, pt3.0, pt3.1, pt1.0, pt1.1, color, thickness, mask);
}

pub fn draw_circle_pixels(img: &mut RgbaImage, cx: i32, cy: i32, radius: i32, color: Rgba<u8>, thickness: u32, mask: Option<(u32, u32, u32, u32)>)
{
    let mut x = radius;
    let mut y = 0;
    let mut err = 0;
    
    let brush_r = (thickness as i32 - 1) / 2;

    while x >= y
    {
        if thickness <= 1
        {
            let points = [
                (cx + x, cy + y), (cx + y, cy + x), (cx - y, cy + x), (cx - x, cy + y),
                (cx - x, cy - y), (cx - y, cy - x), (cx + y, cy - x), (cx + x, cy - y)
            ];
            for (px, py) in points
            {
                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32
                {
                    put_pixel_masked(img, px as u32, py as u32, color, mask);
                }
            }
        }
        else
        {
            let points = [
                (cx + x, cy + y), (cx + y, cy + x), (cx - y, cy + x), (cx - x, cy + y),
                (cx - x, cy - y), (cx - y, cy - x), (cx + y, cy - x), (cx + x, cy - y)
            ];
            for (px, py) in points
            {
                draw_filled_circle(img, px, py, brush_r, color, mask);
            }
        }

        if err <= 0
        {
            y += 1;
            err += 2 * y + 1;
        }
        if err > 0
        {
            x -= 1;
            err -= 2 * x + 1;
        }
    }
}
