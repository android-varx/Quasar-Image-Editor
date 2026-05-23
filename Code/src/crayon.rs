use image::{Rgba, RgbaImage};

fn put_pixel_masked(img: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>, mask: Option<(u32, u32, u32, u32)>) {
    if let Some((mx0, my0, mx1, my1)) = mask {
        if x < mx0 || x > mx1 || y < my0 || y > my1 {
            return;
        }
    }
    img.put_pixel(x, y, color);
}

pub fn draw_filled_circle(img: &mut RgbaImage, cx: i32, cy: i32, radius: i32, color: Rgba<u8>, mask: Option<(u32, u32, u32, u32)>)
{
    let r2 = radius * radius;
    for y in -radius..=radius
    {
        for x in -radius..=radius
        {
            if x * x + y * y <= r2
            {
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32
                {
                    put_pixel_masked(img, px as u32, py as u32, color, mask);
                }
            }
        }
    }
}

pub fn draw_line(img: &mut RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgba<u8>, thickness: u32, mask: Option<(u32, u32, u32, u32)>)
{
    let mut x0 = x0 as i32;
    let mut y0 = y0 as i32;
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let radius = (thickness as i32 - 1) / 2;

    let dx = (x1 - x0).abs();
    
    let sx = if x0 < x1
    {
        1
    }
    else
    {
        -1
    };
    
    let dy = -(y1 - y0).abs();
    
    let sy = if y0 < y1
    {
        1
    }
    else
    {
        -1
    };
    
    let mut err = dx + dy;

    loop
    {
        if thickness <= 1
        {
            if x0 >= 0 && x0 < img.width() as i32 && y0 >= 0 && y0 < img.height() as i32
            {
                put_pixel_masked(img, x0 as u32, y0 as u32, color, mask);
            }
        }
        else
        {
            draw_filled_circle(img, x0, y0, radius, color, mask);
        }

        if x0 == x1 && y0 == y1
        {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy
        {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx
        {
            err += dx;
            y0 += sy;
        }
    }
}

pub fn erase_filled_circle(img: &mut RgbaImage, orig: &RgbaImage, cx: i32, cy: i32, radius: i32, mask: Option<(u32, u32, u32, u32)>)
{
    let r2 = radius * radius;
    for y in -radius..=radius
    {
        for x in -radius..=radius
        {
            if x * x + y * y <= r2
            {
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 && px < orig.width() as i32 && py < orig.height() as i32
                {
                    let orig_color = *orig.get_pixel(px as u32, py as u32);
                    put_pixel_masked(img, px as u32, py as u32, orig_color, mask);
                }
            }
        }
    }
}

pub fn erase_line(img: &mut RgbaImage, orig: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, thickness: u32, mask: Option<(u32, u32, u32, u32)>)
{
    let mut x0 = x0 as i32;
    let mut y0 = y0 as i32;
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let radius = (thickness as i32 - 1) / 2;

    let dx = (x1 - x0).abs();
    
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    
    let mut err = dx + dy;

    loop
    {
        if thickness <= 1
        {
            if x0 >= 0 && x0 < img.width() as i32 && y0 >= 0 && y0 < img.height() as i32 && x0 < orig.width() as i32 && y0 < orig.height() as i32
            {
                let orig_color = *orig.get_pixel(x0 as u32, y0 as u32);
                put_pixel_masked(img, x0 as u32, y0 as u32, orig_color, mask);
            }
        }
        else
        {
            erase_filled_circle(img, orig, x0, y0, radius, mask);
        }

        if x0 == x1 && y0 == y1
        {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy
        {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx
        {
            err += dx;
            y0 += sy;
        }
    }
}
