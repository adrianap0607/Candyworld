use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub hit_x: usize,
    pub hit_y: usize,
}

#[inline]
fn is_wall(c: char) -> bool {
    !(c == ' ' || c == 'g' || c == 'p' || c == 'b' || c == 'V')
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    let mut d: f32 = 0.0;
    const STEP: f32 = 2.0;

    framebuffer.set_current_color(Color::WHITESMOKE);

    loop {
        let x_f = player.pos.x + d * a.cos();
        let y_f = player.pos.y + d * a.sin();
        let xi = x_f as isize;
        let yi = y_f as isize;

        if xi < 0 || yi < 0 {
            let hx = if xi < 0 { 0 } else { xi as usize };
            let hy = if yi < 0 { 0 } else { yi as usize };
            return Intersect { distance: d.max(1.0), impact: '#', hit_x: hx, hit_y: hy };
        }

        let x = xi as usize;
        let y = yi as usize;

        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return Intersect { distance: d.max(1.0), impact: '#', hit_x: x, hit_y: y };
        }

        let tile = maze[j][i];
        if is_wall(tile) {
            return Intersect { distance: d.max(1.0), impact: tile, hit_x: x, hit_y: y };
        }

        if draw_line { framebuffer.set_pixel(x as u32, y as u32); }
        d += STEP;
    }
}