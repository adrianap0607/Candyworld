use raylib::prelude::{RaylibAudio, Sound};
use std::cell::RefCell;

pub struct SoundManager {
    audio: &'static RaylibAudio,
    piece: Sound<'static>,
    music: RefCell<Sound<'static>>,
}

impl SoundManager {
    pub fn new() -> Self {
        let audio: &'static RaylibAudio = Box::leak(Box::new(
            RaylibAudio::init_audio_device().expect("No se pudo inicializar audio"),
        ));

        // Sonido  al recoger
        let piece = audio
            .new_sound("sounds/piece.mp3")
            .expect("No se pudo cargar sounds/piece.mp3");

        // Música de fondo
        let mut music = audio
            .new_sound("sounds/candy.mp3")
            .expect("No se pudo cargar sounds/candy.mp3");
        music.set_volume(0.7);

        let mgr = SoundManager {
            audio,
            piece,
            music: RefCell::new(music),
        };

        mgr.start_music();
        mgr
    }

    pub fn play_piece(&self) {
        self.piece.play();
    }

    pub fn start_music(&self) {
        let mut m = self.music.borrow_mut();
        if !m.is_playing() {
            m.play();
        }
    }

    pub fn update(&self) {
        let mut m = self.music.borrow_mut();
        if !m.is_playing() {
            m.play();
        }
    }

    pub fn set_music_volume(&self, v: f32) {
        self.music.borrow_mut().set_volume(v.clamp(0.0, 1.0));
    }
}