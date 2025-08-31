use raylib::prelude::*;
use std::collections::HashMap;

pub struct TextureManager {
    images: HashMap<char, Image>,
    textures: HashMap<char, Texture2D>,
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut images = HashMap::new();
        let mut textures = HashMap::new();

        let texture_files = vec![
            ('+', "assets/texture2.jpg"),
            ('-', "assets/texture3.jpg"),
            ('|', "assets/texture4.png"),
            ('g', "assets/texture5.jpg"),
            ('#', "assets/texture5.jpg"),
            ('b', "assets/donut.png"), 
            ('V', "assets/donut.png"),
        ];

        for (ch, path) in texture_files {
            if let Ok(image) = Image::load_image(path) {
                images.insert(ch, image);
                if let Ok(tex) = rl.load_texture(thread, path) {
                    textures.insert(ch, tex);
                }
            } else {
                eprintln!("Failed to load image {}", path);
            }
        }

        TextureManager { images, textures }
    }

    pub fn get_pixel_color(&mut self, ch: char, tx: u32, ty: u32) -> Color {
        if let Some(image) = self.images.get_mut(&ch) {
            let x = tx.min(image.width.max(1) as u32 - 1) as i32;
            let y = ty.min(image.height.max(1) as u32 - 1) as i32;
            image.get_color(x, y)
        } else {
            Color::WHITE
        }
    }

    pub fn sample_uv(&mut self, ch: char, u: f32, v: f32) -> Color {
        if let Some(image) = self.images.get_mut(&ch) {
            let w = image.width.max(1) as f32;
            let h = image.height.max(1) as f32;
            let uu = u.fract().abs().clamp(0.0, 1.0);
            let vv = v.fract().abs().clamp(0.0, 1.0);
            let tx = (uu * (w - 1.0)).round() as u32;
            let ty = (((1.0 - vv) * (h - 1.0)).round() as u32).min((h - 1.0) as u32);
            self.get_pixel_color(ch, tx, ty)
        } else {
            Color::WHITE
        }
    }

    pub fn get_texture(&self, ch: char) -> Option<&Texture2D> {
        self.textures.get(&ch)
    }
}

impl TextureManager {
    pub fn image_size(&self, ch: char) -> (u32, u32) {
        if let Some(img) = self.images.get(&ch) {
            (img.width.max(1) as u32, img.height.max(1) as u32)
        } else {
            (128, 128)
        }
    }
}