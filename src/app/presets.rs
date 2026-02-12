use super::{App, CurrentView};

impl App {
    pub fn confirm_preset_input(&mut self) {
        let name = self.preset_input_buffer.trim().to_string();
        if name.is_empty() {
            return;
        }

        if let Some(index) = self.preset_rename_target {
            // Rename existing
            if let Some(preset) = self.presets_config.presets.get_mut(index) {
                preset.name = name;
            }
        } else {
            // Create new
            let mut preset_sounds = std::collections::HashMap::new();

            // Capture currently playing sounds
            if let Some(engine) = &self.audio_engine {
                for sound in &self.sounds {
                    if engine.is_playing(&sound.id) {
                        preset_sounds.insert(sound.id.clone(), sound.volume_linear);
                    }
                }
            }

            let new_preset = crate::presets::Preset {
                name,
                sounds: preset_sounds,
            };

            self.presets_config.presets.push(new_preset);
        }

        let _ = self.presets_config.save();
        self.preset_rename_target = None;
    }

    pub fn start_renaming_preset(&mut self) {
        if let Some(preset) = self.presets_config.presets.get(self.preset_cursor_pos) {
            self.preset_input_buffer = preset.name.clone();
            self.preset_rename_target = Some(self.preset_cursor_pos);
            self.preset_input_mode = true;
        }
    }

    pub fn update_preset_sounds(&mut self) {
        if self.preset_cursor_pos >= self.presets_config.presets.len() {
            return;
        }

        let mut preset_sounds = std::collections::HashMap::new();
        if let Some(engine) = &self.audio_engine {
            for sound in &self.sounds {
                if engine.is_playing(&sound.id) {
                    preset_sounds.insert(sound.id.clone(), sound.volume_linear);
                }
            }
        }

        if let Some(preset) = self.presets_config.presets.get_mut(self.preset_cursor_pos) {
            preset.sounds = preset_sounds;
        }
        let _ = self.presets_config.save();
    }

    pub fn load_preset(&mut self, index: usize) {
        if index >= self.presets_config.presets.len() {
            return;
        }

        // Clone the sounds map to avoid borrowing self while mutating self later
        let preset_sounds = self.presets_config.presets[index].sounds.clone();

        self.stop_all();

        // Need to update app.sounds volumes and play them
        if let Some(engine) = &mut self.audio_engine {
            for sound in &mut self.sounds {
                if let Some(&vol) = preset_sounds.get(&sound.id) {
                    sound.volume_linear = vol;
                    sound.error_state = false;
                    if let Err(e) = engine.play(&sound.id, &sound.file_path, sound.volume_linear) {
                        log::error!("Failed to play preset sound '{}': {}", sound.id, e);
                        sound.error_state = true;
                    }
                } else {
                    // Optionally reset volume of unused sounds or keep them as is?
                    // Usually presets define the active set.
                }
            }
        }
        self.active_preset = Some(self.presets_config.presets[index].name.clone());
        self.view = CurrentView::Main;
    }

    pub fn delete_preset(&mut self, index: usize) {
        if index < self.presets_config.presets.len() {
            self.presets_config.presets.remove(index);
            let _ = self.presets_config.save();
            if self.preset_cursor_pos >= self.presets_config.presets.len()
                && !self.presets_config.presets.is_empty()
            {
                self.preset_cursor_pos = self.presets_config.presets.len() - 1;
            }
        }
    }
}
