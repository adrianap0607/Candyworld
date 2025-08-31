use raylib::prelude::*;
use std::f32::consts::PI;

use crate::{framebuffer::Framebuffer, player::Player, texture::TextureManager};

pub const TRANSPARENT_COLOR: Color = Color::new(152, 0, 136, 255);

#[derive(Clone)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub kind: char,
    pub collected: bool,
}

impl Sprite {
    pub fn new_cell(i: usize, j: usize, block: usize, kind: char) -> Self {
        let x = (i * block + block / 2) as f32;
        let y = (j * block + block / 2) as f32;
        Self { x, y, kind, collected: false }
    }
}

#[inline]
fn is_chroma(c: Color) -> bool {
    let dr = c.r.abs_diff(TRANSPARENT_COLOR.r);
    let dg = c.g.abs_diff(TRANSPARENT_COLOR.g);
    let db = c.b.abs_diff(TRANSPARENT_COLOR.b);
    (dr as u16 + dg as u16 + db as u16) <= 3 && c.a == TRANSPARENT_COLOR.a
}

pub fn draw_sprite(
    fb: &mut Framebuffer,
    player: &Player,
    sprite: &Sprite,
    texman: &mut TextureManager,
    depth_buffer: &[f32],
    block_size: usize,
) {
    let dx = sprite.x - player.pos.x;
    let dy = sprite.y - player.pos.y;

    let angle_to = dy.atan2(dx);

    let mut ang = angle_to - player.a;
    while ang >  PI { ang -= 2.0 * PI; }
    while ang < -PI { ang += 2.0 * PI; }

    if ang.abs() > player.fov * 0.5 { return; }

    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1.0 { return; }

    let corr = (dist * ang.cos()).max(0.001);

    let proj_plane = (fb.width as f32 / 2.0) / (player.fov * 0.5).tan();
    let scale = 0.5;
    let sprite_h = (block_size as f32 * proj_plane * scale) / corr;
    let sprite_w = sprite_h;

    let screen_cx = (fb.width as f32 / 2.0) + ang.tan() * proj_plane;

    let x0 = ((screen_cx - sprite_w * 0.5).round() as i32).max(0) as usize;
    let x1 = ((screen_cx + sprite_w * 0.5).round() as i32)
        .min(fb.width as i32 - 1) as usize;

    let y0 = ((fb.height as f32 / 2.0 - sprite_h * 0.5).round() as i32).max(0) as usize;
    let y1 = ((fb.height as f32 / 2.0 + sprite_h * 0.5).round() as i32)
        .min(fb.height as i32 - 1) as usize;

    if x1 <= x0 || y1 <= y0 { return; }

    let (tw, th) = texman.image_size(sprite.kind);

    for sx in x0..=x1 {
        let col = sx as usize;
        if col < depth_buffer.len() {
            if corr > depth_buffer[col] {
                continue;
            }
        }

        let u = (sx as f32 - (screen_cx - sprite_w * 0.5)) / sprite_w;
        let tx = ((1.0 - u.clamp(0.0, 1.0)) * (tw.saturating_sub(1) as f32)).round() as u32;

        for sy in y0..=y1 {
            let v  = (sy as f32 - (fb.height as f32 / 2.0 - sprite_h * 0.5)) / sprite_h;
            let ty = (v.clamp(0.0, 1.0) * (th.saturating_sub(1) as f32)).round() as u32;

            let c = texman.get_pixel_color(sprite.kind, tx, ty);

            if c.a == 0 { continue; }

            if is_chroma(c) { continue; }

            fb.set_current_color(c);
            fb.set_pixel(sx as u32, sy as u32);
        }
    }
}