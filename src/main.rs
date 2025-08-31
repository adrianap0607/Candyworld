mod framebuffer;
mod line;
mod caster;
mod texture;
mod maze;
mod player;
mod sprite;

use raylib::prelude::*;
use std::f32::consts::PI;

use crate::framebuffer::Framebuffer;
use crate::caster::cast_ray;
use crate::texture::TextureManager;
use crate::maze::{Maze, load_maze, find_char};
use crate::player::{Player, process_events};
use crate::sprite::{Sprite, draw_sprite};

#[derive(Clone, Copy)]
struct LevelSpec { file: &'static str, time_limit: i32 }

static LEVELS: &[LevelSpec] = &[
    LevelSpec { file: "maze.txt",  time_limit: 60 },
    LevelSpec { file: "maze2.txt", time_limit: 60 },
];

fn load_sprites_from_maze(maze: &Maze, block: usize) -> Vec<Sprite> {
    let mut v = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            match c {
                'b' | 'V' => v.push(Sprite::new_cell(i, j, block, c)),
                _ => {}
            }
        }
    }
    v
}

fn show_success_screen(
    rl: &mut RaylibHandle,
    th: &RaylibThread,
    tex: &Texture2D,
    screen_w: i32,
    screen_h: i32,
) {
    loop {
        {
            let mut d = rl.begin_drawing(th);
            d.clear_background(Color::BLACK);

            let scale = (screen_w as f32 / tex.width as f32)
                .max(screen_h as f32 / tex.height as f32);
            let dest_w = tex.width as f32 * scale;
            let dest_h = tex.height as f32 * scale;

            let src = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
            let dest = Rectangle::new(
                (screen_w as f32 - dest_w) * 0.5,
                (screen_h as f32 - dest_h) * 0.5,
                dest_w,
                dest_h,
            );
            d.draw_texture_pro(tex, src, dest, Vector2::zero(), 0.0, Color::WHITE);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.window_should_close() { break; }
    }
}

fn show_lost_screen(
    rl: &mut RaylibHandle,
    th: &RaylibThread,
    tex: &Texture2D,
    screen_w: i32,
    screen_h: i32,
) {
    loop {
        {
            let mut d = rl.begin_drawing(th);
            d.clear_background(Color::BLACK);

            let scale = (screen_w as f32 / tex.width as f32)
                .max(screen_h as f32 / tex.height as f32);
            let dest_w = tex.width as f32 * scale;
            let dest_h = tex.height as f32 * scale;

            let src = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
            let dest = Rectangle::new(
                (screen_w as f32 - dest_w) * 0.5,
                (screen_h as f32 - dest_h) * 0.5,
                dest_w,
                dest_h,
            );
            d.draw_texture_pro(tex, src, dest, Vector2::zero(), 0.0, Color::WHITE);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) || rl.window_should_close() { break; }
    }
}

fn show_start_screen(
    rl: &mut RaylibHandle,
    th: &RaylibThread,
    tex: &Texture2D,
    screen_w: i32,
    screen_h: i32,
) {
    loop {
        {
            let mut d = rl.begin_drawing(th);
            d.clear_background(Color::BLACK);

            let scale = (screen_w as f32 / tex.width as f32)
                .max(screen_h as f32 / tex.height as f32);
            let dest_w = tex.width as f32 * scale;
            let dest_h = tex.height as f32 * scale;

            let src = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
            let dest = Rectangle::new(
                (screen_w as f32 - dest_w) * 0.5,
                (screen_h as f32 - dest_h) * 0.5,
                dest_w,
                dest_h,
            );
            d.draw_texture_pro(tex, src, dest, Vector2::zero(), 0.0, Color::WHITE);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.window_should_close() { break; }
    }
}

#[derive(Clone)]
struct Candy { i: usize, j: usize, collected: bool }

struct Level { number: u32, candies_needed: u32, duration_secs: i32 }

struct GameState {
    level: Level,
    candies: Vec<Candy>,
    level_deadline: f64,
    paused: bool,
    msg_text: Option<String>,
    msg_until: f64,
    pause_started: f64,
    pending_action: Option<PendingAction>,
}

enum PendingAction { NextLevel, RestartLevel }

fn spawn_candies(maze: &Maze, n: u32) -> Vec<Candy> {
    let mut spaces = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == ' ' || c == 'g' { spaces.push((i, j)); }
        }
    }
    let mut out = Vec::new();
    if spaces.is_empty() { return out; }
    let step = (spaces.len() as u32 / n.max(1)).max(1);
    let mut idx = step / 2;
    while out.len() < n as usize {
        let (i, j) = spaces[(idx as usize) % spaces.len()];
        if !out.iter().any(|c| c.i == i && c.j == j) {
            out.push(Candy { i, j, collected: false });
        }
        idx = idx.wrapping_add(step);
    }
    out
}

fn player_cell(player: &Player, block_size: usize) -> (usize, usize) {
    ((player.pos.x as usize) / block_size, (player.pos.y as usize) / block_size)
}

fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    depth_buffer: &mut [f32],
    texman: &mut TextureManager,
) {
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let hh = framebuffer.height as f32 / 2.0;
    let dist_proj_plane = hw / (player.fov * 0.5).tan();

    assert_eq!(depth_buffer.len(), num_rays as usize);

    for i in 0..num_rays {
        let t = i as f32 / num_rays as f32;
        let a = player.a - (player.fov * 0.5) + (player.fov * t);

        let hit = cast_ray(framebuffer, maze, player, a, block_size, false);
        let corrected = (hit.distance * (a - player.a).cos()).max(1.0);
        depth_buffer[i as usize] = corrected;

        let stake_h = (block_size as f32 * dist_proj_plane) / corrected;
        let top = (hh - stake_h * 0.5).max(0.0) as u32;
        let bot = (hh + stake_h * 0.5).min(framebuffer.height as f32 - 1.0) as u32;

        let sky_col   = Color::new(0xC7, 0xD9, 0xDD, 255);
        let floor_col = Color::new(255, 170, 170, 255);

        framebuffer.set_current_color(sky_col);
        for y in 0..top { framebuffer.set_pixel(i, y); }

        let fx = (hit.hit_x as f32) / (block_size as f32);
        let fy = (hit.hit_y as f32) / (block_size as f32);
        let frac_x = fx - fx.floor();
        let frac_y = fy - fy.floor();
        let near_edge_x = frac_x < 0.001 || frac_x > 0.999;
        let near_edge_y = frac_y < 0.001 || frac_y > 0.999;

        let u = if near_edge_x && !near_edge_y { frac_y }
                else if near_edge_y && !near_edge_x { frac_x }
                else { if (frac_x - 0.5).abs() > (frac_y - 0.5).abs() { frac_y } else { frac_x } };

        let ch = match hit.impact { '+' | '-' | '|' | 'g' => hit.impact, _ => '#' };

        for y in top..=bot {
            let v = (y as f32 - top as f32) / (bot.saturating_sub(top).max(1) as f32);
            let mut color = texman.sample_uv(ch, u, v);
            let shade = (1.0 / (1.0 + 0.0015 * corrected)).clamp(0.60, 1.0);
            color.r = ((color.r as f32) * shade) as u8;
            color.g = ((color.g as f32) * shade) as u8;
            color.b = ((color.b as f32) * shade) as u8;
            framebuffer.set_current_color(color);
            framebuffer.set_pixel(i, y);
        }

        framebuffer.set_current_color(floor_col);
        for y in (bot+1)..(framebuffer.height) { framebuffer.set_pixel(i, y); }
    }
}

fn render_minimap(
    fb: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    sprites: &[Sprite],
) {
    let mini = 8usize;
    let ox = 10usize;
    let oy = 10usize;

    let w = maze[0].len() * mini;
    let h = maze.len() * mini;
    fb.set_current_color(Color::new(20, 20, 30, 255));
    for x in ox..ox + w { for y in oy..oy + h { fb.set_pixel(x as u32, y as u32); } }

    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == ' ' { continue; }
            let x0 = ox + i * mini;
            let y0 = oy + j * mini;
            let col = match c { '+'|'-'|'|' => Color::DARKPURPLE, 'g' => Color::GREEN, _ => Color::GRAY };
            fb.set_current_color(col);
            for x in x0..x0 + mini { for y in y0..y0 + mini { fb.set_pixel(x as u32, y as u32); } }
        }
    }

    for s in sprites.iter().filter(|s| !s.collected) {
        let i = (s.x as usize) / block_size;
        let j = (s.y as usize) / block_size;
        let x0 = ox + i * mini;
        let y0 = oy + j * mini;
        fb.set_current_color(Color::GRAY);
        for x in x0..x0 + mini { for y in y0..y0 + mini { fb.set_pixel(x as u32, y as u32); } }
    }

    let scale = mini as f32 / block_size as f32;
    let jx = ox as f32 + player.pos.x * scale;
    let jy = oy as f32 + player.pos.y * scale;
    fb.draw_circle_filled(jx.round() as i32, jy.round() as i32, 4, Color::RED);
}

fn load_level(
    idx: usize,
    block_size: usize,
    rl: &RaylibHandle,
) -> (Maze, Vec<Sprite>, Vector2, f64) {
    let mut maze = load_maze(LEVELS[idx].file);

    let (pi, pj) = find_char(&maze, 'p').unwrap_or((1, 1));
    let spawn = Vector2::new(
        (pi * block_size + block_size / 2) as f32,
        (pj * block_size + block_size / 2) as f32,
    );

    let sprites = load_sprites_from_maze(&maze, block_size);

    for row in maze.iter_mut() {
        for c in row.iter_mut() {
            if matches!(*c, 'p' | 'V' | 'b' | '1' | '2' | '3') { *c = ' '; }
        }
    }

    let deadline = rl.get_time() + LEVELS[idx].time_limit as f64;
    (maze, sprites, spawn, deadline)
}

fn main() {
    const SCREEN_W: u32 = 800;
    const SCREEN_H: u32 = 600;
    const BLOCK_SIZE: usize = 64;

    let (mut rl, raylib_thread) = raylib::init()
        .size(SCREEN_W as i32, SCREEN_H as i32)
        .title("Candy Maze")
        .build();

    let exito_tex: Texture2D = rl
        .load_texture(&raylib_thread, "assets/exito.png")
        .expect("No se pudo cargar assets/exito.png");

    let inicio_tex: Texture2D = rl
        .load_texture(&raylib_thread, "assets/inicio.png")
        .expect("No se pudo cargar assets/inicio.png");
    show_start_screen(&mut rl, &raylib_thread, &inicio_tex, SCREEN_W as i32, SCREEN_H as i32);

    let lost_tex: Texture2D = rl
        .load_texture(&raylib_thread, "assets/lost.png")
        .expect("No se pudo cargar assets/lost.png");

    let mut framebuffer = Framebuffer::new(SCREEN_W, SCREEN_H);

    let mut current_level: usize = 0;
    let (mut maze, mut sprites, spawn, mut deadline) = load_level(current_level, BLOCK_SIZE, &rl);

    let mut player = Player { pos: spawn, a: -PI / 2.0, fov: PI / 3.0 };

    let mut texman = TextureManager::new(&mut rl, &raylib_thread);

    let mut state = GameState {
        level: Level { number: 1, candies_needed: 5, duration_secs: 60 },
        candies: Vec::new(),
        level_deadline: deadline,
        paused: false,
        msg_text: None,
        msg_until: 0.0,
        pause_started: 0.0,
        pending_action: None,
    };

    let mut depth_buffer: Vec<f32> = vec![f32::INFINITY; framebuffer.width as usize];

    while !rl.window_should_close() {
        framebuffer.clear();

        let now = rl.get_time();
        if state.paused {
            if now >= state.msg_until {
                let paused_delta = now - state.pause_started;
                state.level_deadline += paused_delta;
                state.paused = false;
                state.msg_text = None;

                if let Some(action) = state.pending_action.take() {
                    match action {
                        PendingAction::NextLevel => {
                            current_level = (current_level + 1) % LEVELS.len();
                            let (m2, s2, spawn2, dl2) = load_level(current_level, BLOCK_SIZE, &rl);
                            maze = m2; sprites = s2; state.level_deadline = dl2;
                            player.pos = spawn2; player.a = -PI/2.0;
                        }
                        PendingAction::RestartLevel => {
                            let (m2, s2, spawn2, dl2) = load_level(current_level, BLOCK_SIZE, &rl);
                            maze = m2; sprites = s2; state.level_deadline = dl2;
                            player.pos = spawn2; player.a = -PI/2.0;
                        }
                    }
                }
            }
        } else {
            process_events(&mut player, &rl, &maze, BLOCK_SIZE);

            let (ci, cj) = player_cell(&player, BLOCK_SIZE);
            for s in &mut sprites {
                let si = (s.x as usize) / BLOCK_SIZE;
                let sj = (s.y as usize) / BLOCK_SIZE;
                if !s.collected && si == ci && sj == cj { s.collected = true; }
            }

            let total_sprites = sprites.len() as u32;
            let collected = sprites.iter().filter(|s| s.collected).count() as u32;
            let remaining = (state.level_deadline - now).ceil() as i32;

            if total_sprites > 0 && collected >= total_sprites && remaining >= 0 {
                show_success_screen(&mut rl, &raylib_thread, &exito_tex, SCREEN_W as i32, SCREEN_H as i32);

                if current_level + 1 >= LEVELS.len() {
                    show_start_screen(&mut rl, &raylib_thread, &inicio_tex, SCREEN_W as i32, SCREEN_H as i32);
                    current_level = 0;
                    let (m2, s2, spawn2, dl2) = load_level(current_level, BLOCK_SIZE, &rl);
                    maze = m2; sprites = s2; state.level_deadline = dl2;
                    player.pos = spawn2; player.a = -PI/2.0;
                } else {
                    current_level = (current_level + 1) % LEVELS.len();
                    let (m2, s2, spawn2, dl2) = load_level(current_level, BLOCK_SIZE, &rl);
                    maze = m2; sprites = s2; state.level_deadline = dl2;
                    player.pos = spawn2; player.a = -PI/2.0;
                }
            } else if remaining < 0 {
                show_lost_screen(&mut rl, &raylib_thread, &lost_tex, SCREEN_W as i32, SCREEN_H as i32);

                let (m2, s2, spawn2, dl2) = load_level(current_level, BLOCK_SIZE, &rl);
                maze = m2; sprites = s2; state.level_deadline = dl2;
                player.pos = spawn2; player.a = -PI/2.0;
            }
        }

        render_world(&mut framebuffer, &maze, BLOCK_SIZE, &player, &mut depth_buffer, &mut texman);

        for s in sprites.iter().filter(|s| !s.collected) {
            draw_sprite(&mut framebuffer, &player, s, &mut texman, &depth_buffer, BLOCK_SIZE);
        }

        render_minimap(&mut framebuffer, &maze, BLOCK_SIZE, &player, &sprites);

        let hud = Some((
            sprites.iter().filter(|s| s.collected).count() as u32,
            sprites.len() as u32,
            (state.level_deadline - rl.get_time()).ceil() as i32,
        ));

        framebuffer.swap_buffers_with_hud(&mut rl, &raylib_thread, hud, state.msg_text.as_deref());

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}