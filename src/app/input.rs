use super::{App, CurrentView};
use crate::static_data::Sound;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

impl App {
    pub fn get_sound_at_pos(&self, x: u16, y: u16) -> Option<(usize, bool, f32)> {
        // Returns (index, is_slider, slider_value)
        let mut current_y = 3i32 - self.grid_scroll as i32;
        let card_width = 24;
        let card_height = 5;
        let col_width = card_width + 2;
        let cols = self.grid_cols;

        let filtered = self.get_filtered_sounds();
        let mut categories: Vec<String> =
            filtered.iter().map(|(_, s)| s.category.clone()).collect();
        categories.dedup();

        for category in categories {
            // Optimization: if current_y is way past y, we can break?
            // Not strictly, as categories follow each other.
            // If current_y > y, and we are not in this block, we might have passed it?
            // Since y is absolute screen coord, and current_y is screen coord.

            if current_y + 1 > (self.height - 3) as i32 {
                break;
            }
            current_y += 2; // Header

            let sounds_in_cat: Vec<(usize, &Sound)> = filtered
                .iter()
                .filter(|(_, s)| s.category == category)
                .map(|(i, s)| (*i, *s))
                .collect();

            for chunk in sounds_in_cat.chunks(cols as usize) {
                // Bounds check for chunk row
                if (y as i32) >= current_y && (y as i32) < current_y + card_height {
                    for (i, (global_idx, _)) in chunk.iter().enumerate() {
                        let card_x = 2 + (i as u16 * col_width);
                        if card_x + card_width > self.width {
                            continue;
                        }

                        if x >= card_x && x < card_x + card_width {
                            // Hit!
                            let rel_y = y as i32 - current_y;
                            // 3 is slider row
                            if rel_y == 3 {
                                let slider_start = card_x + 2;
                                let slider_visual_width = 12;
                                let touch_width = slider_visual_width + 6;
                                if x >= slider_start && x < slider_start + touch_width {
                                    let rel_x = x - slider_start;
                                    let vol = (rel_x as f32 / (slider_visual_width as f32 - 1.0))
                                        .clamp(0.0, 1.0);
                                    return Some((*global_idx, true, vol));
                                }
                                // Hit card row 3 but not slider
                                return Some((*global_idx, false, 0.0));
                            } else {
                                return Some((*global_idx, false, 0.0));
                            }
                        }
                    }
                }
                current_y += card_height + 1;
            }
        }
        None
    }

    pub fn handle_mouse_event(&mut self, event: MouseEvent) {
        let x = event.column;
        let y = event.row;
        let kind = event.kind;

        // Header Tab Click Handling
        if y < 3 {
            self.handle_header_interaction(x, kind);
            return;
        }

        // Footer (Master Volume)
        if y == self.height - 2 {
            self.handle_footer_interaction(x, kind);
            return;
        }

        // View Specifics
        match self.view {
            CurrentView::Presets => self.handle_preset_interaction(y, kind),
            CurrentView::Main => self.handle_grid_interaction(x, y, kind),
            _ => {}
        }
    }

    pub fn handle_header_interaction(&mut self, x: u16, kind: MouseEventKind) {
        // If searching, header tabs are not visible, so disable interaction
        if self.search_mode || !self.search_query.is_empty() {
            return;
        }

        if matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
            // Layout defines first chunk as Length(20)
            let tabs_start_x = 20;
            if x >= tabs_start_x {
                let rel_x = x - tabs_start_x;
                // " Sounds " is 8 chars
                if rel_x < 8 {
                    self.view = CurrentView::Main;
                }
                // Separator is 1 char
                // " Presets " is 9 chars. Starts at 8+1=9. Ends at 9+9=18.
                else if (9..18).contains(&rel_x) {
                    self.view = CurrentView::Presets;
                }
                // " Downloads " is 11 chars. Starts at 18+1=19. Ends at 19+11=30.
                else if self.yt_dlp_available && (19..30).contains(&rel_x) {
                    self.view = CurrentView::Downloads;
                }
            }

            // Check for Play/Mute button on right
            if x >= self.width.saturating_sub(20) {
                let rel_x = x - self.width.saturating_sub(20);
                if rel_x < 13 {
                    self.toggle_mute();
                }
            }
        }
    }

    pub fn handle_footer_interaction(&mut self, x: u16, kind: MouseEventKind) {
        match kind {
            MouseEventKind::ScrollUp => {
                self.set_master_volume(self.session.global_volume + 0.05);
            }
            MouseEventKind::ScrollDown => {
                self.set_master_volume(self.session.global_volume - 0.05);
            }
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                let prefix_len = if self.muted { 16 } else { 10 };
                let slider_width = 12;

                if x >= prefix_len && x < prefix_len + slider_width {
                    let relative_x = x - prefix_len;
                    let vol = relative_x as f32 / (slider_width - 1) as f32;
                    self.set_master_volume(vol);
                    return;
                }

                if matches!(kind, MouseEventKind::Down(MouseButton::Left)) && x < prefix_len {
                    self.toggle_mute();
                }
            }
            _ => {}
        }
    }

    pub fn handle_preset_interaction(&mut self, y: u16, kind: MouseEventKind) {
        if y >= 7 && y < self.height - 3 {
            // Header(3) + Input(3) + Border(1) = 7
            let list_start_y = 7;
            // List Content (inside borders) = H - 11 (approx)
            let list_content_height = (self.height as usize).saturating_sub(11);

            let clicked_row = (y - list_start_y) as usize;

            if clicked_row < list_content_height {
                let offset = if self.preset_cursor_pos >= list_content_height {
                    self.preset_cursor_pos - list_content_height + 1
                } else {
                    0
                };

                let target_idx = offset + clicked_row;

                if target_idx < self.presets_config.presets.len() {
                    if let MouseEventKind::Down(MouseButton::Left) = kind {
                        if self.preset_cursor_pos == target_idx {
                            // Double click / second click -> Load
                            self.load_preset(target_idx);
                        } else {
                            self.preset_cursor_pos = target_idx;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_grid_interaction(&mut self, x: u16, y: u16, kind: MouseEventKind) {
        if y >= 3 && y < self.height - 3 {
            let hit = self.get_sound_at_pos(x, y);

            match kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some((idx, _, slider_val)) = hit {
                        self.cursor_pos = idx;
                        if let Some((_, is_slider, _)) = hit {
                            if is_slider {
                                self.set_current_volume(slider_val);
                            } else {
                                self.toggle_current_sound();
                            }
                        }
                    }
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    if let Some((idx, is_slider, slider_val)) = hit {
                        self.cursor_pos = idx;
                        if is_slider {
                            self.set_current_volume(slider_val);
                        }
                    }
                }
                MouseEventKind::ScrollUp => {
                    if let Some((idx, _, _)) = hit {
                        if let Some(sound) = self.sounds.get(idx) {
                            let new_vol = (sound.volume_linear + 0.05).clamp(0.0, 1.0);
                            self.cursor_pos = idx;
                            self.set_current_volume(new_vol);
                        }
                    } else {
                        self.scroll_grid(-2);
                    }
                }
                MouseEventKind::ScrollDown => {
                    if let Some((idx, _, _)) = hit {
                        if let Some(sound) = self.sounds.get(idx) {
                            let new_vol = (sound.volume_linear - 0.05).clamp(0.0, 1.0);
                            self.cursor_pos = idx;
                            self.set_current_volume(new_vol);
                        }
                    } else {
                        self.scroll_grid(2);
                    }
                }
                _ => {}
            }
        }
    }
}
