use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        Self {
            width, height, color_buffer,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(
            self.width as i32,
            self.height as i32,
            self.background_color,
        );
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_pixel_with_color_i32(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            self.color_buffer.draw_pixel(x, y, color);
        }
    }

    pub fn draw_circle_filled(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let rr = r * r;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx*dx + dy*dy <= rr {
                    self.set_pixel_with_color_i32(cx + dx, cy + dy, color);
                }
            }
        }
    }

    pub fn set_background_color(&mut self, color: Color) { self.background_color = color; }
    pub fn set_current_color(&mut self, color: Color) { self.current_color = color; }

    pub fn swap_buffers(&self, window: &mut RaylibHandle, rl: &RaylibThread) {
        self.swap_buffers_with_hud(window, rl, None, None);
    }

    pub fn swap_buffers_with_hud(
        &self,
        window: &mut RaylibHandle,
        rl: &RaylibThread,
        hud: Option<(u32, u32, i32)>,
        status: Option<&str>
    ) {
        let fps_val = window.get_fps();
        if let Ok(tex) = window.load_texture_from_image(rl, &self.color_buffer) {
            let mut renderer: RaylibDrawHandle<'_> = window.begin_drawing(rl);
            renderer.clear_background(Color::BLACK);
            renderer.draw_texture(&tex, 0, 0, Color::WHITE);

            if let Some((candies_collected, candies_total, elapsed_secs)) = hud {
                let secs = elapsed_secs.clamp(0, 60);
                let m = secs / 60;
                let s = secs % 60;
                let clock_text = format!("{:02}:{:02}", m, s);
                let candies_text = format!("Candies: {}/{}", candies_collected, candies_total);
                let fps_text = format!("FPS: {}", fps_val);

                let font_size = 20;
                let pad = 10;

                let w_clock   = renderer.measure_text(&clock_text,   font_size);
                let w_candies = renderer.measure_text(&candies_text, font_size);
                let w_fps     = renderer.measure_text(&fps_text,     font_size);
                let block_w = *[w_clock, w_candies, w_fps].iter().max().unwrap_or(&0);

                let mut x = self.width as i32 - block_w - pad;
                let mut y = 10;

                renderer.draw_text(&clock_text,   x, y, font_size, Color::YELLOW);
                y += font_size + 4;
                renderer.draw_text(&candies_text, x, y, font_size, Color::WHITE);
                y += font_size + 4;
                renderer.draw_text(&fps_text,     x, y, font_size, Color::WHITE);
            }

            if let Some(status_text) = status {
                let banner_font_size = 40;
                let text_width = renderer.measure_text(status_text, banner_font_size);
                let text_height = banner_font_size;
                let center_x = (self.width as i32 - text_width) / 2;
                let center_y = (self.height as i32 - text_height) / 2;
                let padding = 20;
                renderer.draw_rectangle(
                    center_x - padding/2,
                    center_y - padding/2,
                    text_width + padding,
                    text_height + padding,
                    Color::new(0, 0, 0, 180)
                );
                renderer.draw_text(
                    status_text,
                    center_x,
                    center_y,
                    banner_font_size,
                    Color::WHITE
                );
            }
        }
    }
}