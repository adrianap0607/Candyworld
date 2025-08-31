// player.rs
use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32,
}

#[inline]
fn is_wall(c: char) -> bool {
    !(c == ' ' || c == 'g' || c == 'b' || c == 'V' || c == 'p')
}

#[inline]
fn tile_at(maze: &Maze, x: f32, y: f32, block_size: usize) -> Option<char> {
    if x < 0.0 || y < 0.0 { return None; }
    let i = (x as usize) / block_size;
    let j = (y as usize) / block_size;
    if j < maze.len() && i < maze[j].len() { Some(maze[j][i]) } else { None }
}

fn can_stand(maze: &Maze, x: f32, y: f32, r: f32, block_size: usize) -> bool {
    let samples = [
        (x - r, y),
        (x + r, y),
        (x, y - r),
        (x, y + r),
        (x - r, y - r),
        (x + r, y - r),
        (x - r, y + r),
        (x + r, y + r),
    ];
    for (sx, sy) in samples {
        match tile_at(maze, sx, sy, block_size) {
            Some(t) => { if is_wall(t) { return false; } }
            None => return false,
        }
    }
    true
}

const PLAYER_RADIUS_FACTOR: f32 = 0.33;
pub fn process_events(player: &mut Player, rl: &RaylibHandle, maze: &Maze, block_size: usize) {
    let dt = rl.get_frame_time() as f32;
    const MOVE_SPEED: f32 = 220.0;
    const ROT_SPEED:  f32 = PI;

    let collision_radius: f32 = (block_size as f32) * PLAYER_RADIUS_FACTOR;

    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a -= ROT_SPEED * dt; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a += ROT_SPEED * dt; }

    let dir = Vector2::new(player.a.cos(), player.a.sin());

    let mut move_vec = Vector2::zero();
    if rl.is_key_down(KeyboardKey::KEY_UP)    || rl.is_key_down(KeyboardKey::KEY_W) { move_vec += dir; }
    if rl.is_key_down(KeyboardKey::KEY_DOWN)  || rl.is_key_down(KeyboardKey::KEY_S) { move_vec -= dir; }
    if rl.is_key_down(KeyboardKey::KEY_A) { move_vec.x += -dir.y; move_vec.y +=  dir.x; }
    if rl.is_key_down(KeyboardKey::KEY_D) { move_vec.x +=  dir.y; move_vec.y += -dir.x; }

    let len = (move_vec.x * move_vec.x + move_vec.y * move_vec.y).sqrt();
    if len > 0.0 {
        let ux = move_vec.x / len;
        let uy = move_vec.y / len;
        let step = MOVE_SPEED * dt;

        let nx = player.pos.x + ux * step;
        if can_stand(maze, nx, player.pos.y, collision_radius, block_size) {
            player.pos.x = nx;
        } else {
            let slip = if ux > 0.0 { 1.0 } else { -1.0 };
            let probe = player.pos.x + slip * 2.0;
            if can_stand(maze, probe, player.pos.y, collision_radius, block_size) {
                player.pos.x = probe;
            }
        }

        let ny = player.pos.y + uy * step;
        if can_stand(maze, player.pos.x, ny, collision_radius, block_size) {
            player.pos.y = ny;
        } else {
            let slip = if uy > 0.0 { 1.0 } else { -1.0 };
            let probe = player.pos.y + slip * 2.0;
            if can_stand(maze, player.pos.x, probe, collision_radius, block_size) {
                player.pos.y = probe;
            }
        }
    }

    if player.a >= 2.0 * PI { player.a -= 2.0 * PI; }
    if player.a < 0.0       { player.a += 2.0 * PI; }
}