use super::App;

impl App {
    pub fn toggle_current_sound(&mut self) {
        if let Some(sound) = self.sounds.get_mut(self.cursor_pos) {
            if let Some(engine) = &mut self.audio_engine {
                if engine.is_playing(&sound.id) {
                    log::info!("Stopping sound '{}'", sound.id);
                    engine.stop(&sound.id);
                } else {
                    log::info!("Starting sound '{}'", sound.id);
                    sound.error_state = false;
                    if let Err(e) = engine.play(&sound.id, &sound.file_path, sound.volume_linear) {
                        log::error!("Failed to play sound '{}': {}", sound.id, e);
                        sound.error_state = true;
                    }
                }
            }
        }
    }

    pub fn set_current_volume(&mut self, vol: f32) {
        if let Some(sound) = self.sounds.get_mut(self.cursor_pos) {
            sound.volume_linear = vol.clamp(0.0, 1.0);
            if let Some(engine) = &mut self.audio_engine {
                engine.set_volume(&sound.id, sound.volume_linear);
            }
        }
    }

    pub fn set_master_volume(&mut self, vol: f32) {
        self.session.global_volume = vol.clamp(0.0, 1.0);
        if let Some(engine) = &mut self.audio_engine {
            engine.set_master_volume(self.session.global_volume);
        }
    }

    pub fn toggle_mute(&mut self) {
        if self.muted {
            self.muted = false;
            self.set_master_volume(self.previous_volume);
        } else {
            self.muted = true;
            self.previous_volume = self.session.global_volume;
            self.set_master_volume(0.0);
        }
    }

    pub fn stop_all(&mut self) {
        if let Some(engine) = &mut self.audio_engine {
            engine.stop_all();
        }
    }
}
