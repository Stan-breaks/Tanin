use super::App;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

pub enum DownloadStatus {
    Pending,
    Downloading(f32),
    Done,
    Error(String),
}

pub struct DownloadTask {
    pub name: String,
    pub category: String,
    pub icon: String,
    pub url: String,
    pub status: DownloadStatus,
}

pub enum DownloadEvent {
    Progress(f32),
    Success(String, String, String, String, String), // name, category, file_path, icon, url
    Error(String),
}

impl App {
    pub fn check_and_download_missing_files(&mut self) {
        if !self.yt_dlp_available {
            return;
        }

        for sound in &self.sounds {
            if let Some(url) = &sound.url {
                let path = std::path::Path::new(&sound.file_path);
                if !path.exists() && !url.trim().is_empty() {
                    // Add to queue
                    self.download_queue.push(DownloadTask {
                        name: sound.name.clone(),
                        category: sound.category.clone(),
                        icon: sound.icon.clone(),
                        url: url.clone(),
                        status: DownloadStatus::Pending,
                    });
                }
            }
        }
    }

    pub fn start_download(&mut self) {
        if self.add_sound_name.trim().is_empty()
            || self.add_sound_category.trim().is_empty()
            || self.add_sound_url.trim().is_empty()
        {
            self.add_sound_status = "Error: All fields (except icon) are required.".to_string();
            return;
        }

        let url = self.add_sound_url.trim().to_string();

        self.download_queue.push(DownloadTask {
            name: self.add_sound_name.clone(),
            category: self.add_sound_category.clone(),
            icon: self.add_sound_icon.clone(),
            url,
            status: DownloadStatus::Pending,
        });

        self.add_sound_status = "Added to download queue.".to_string();
        self.add_sound_name.clear();
        self.add_sound_url.clear();
    }

    pub fn spawn_download_task(&mut self, index: usize) {
        let task = &mut self.download_queue[index];
        task.status = DownloadStatus::Downloading(0.0);
        log::info!("Starting download task for: {}", task.name);

        let (tx, rx) = mpsc::channel();
        self.download_rx = Some(rx);
        self.active_download_index = Some(index);

        let name = task.name.clone();
        let category = task.category.clone();
        let icon = task.icon.clone();
        let url = task.url.clone();

        thread::spawn(move || {
            let proj_dirs = match directories::ProjectDirs::from("com", "tanin", "tanin") {
                Some(dirs) => dirs,
                None => {
                    let _ = tx.send(DownloadEvent::Error(
                        "Could not determine data directory.".to_string(),
                    ));
                    return;
                }
            };

            let sounds_dir = proj_dirs.data_dir().join("sounds");
            if !sounds_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&sounds_dir) {
                    let _ = tx.send(DownloadEvent::Error(format!(
                        "Error creating directory: {}",
                        e
                    )));
                    return;
                }
            }

            let safe_name: String = name
                .trim()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect();

            let output_template = sounds_dir.join(format!("{}.%(ext)s", safe_name));
            let output_template_str = output_template.to_string_lossy();

            log::debug!("Download target: {}", output_template_str);

            let child = std::process::Command::new("yt-dlp")
                .arg("--ignore-config")
                .arg("--no-playlist")
                .arg("--force-overwrites")
                .arg("-x")
                .arg("--audio-format")
                .arg("opus")
                .arg("-f")
                .arg("ba[ext=webm]/ba")
                .arg("-o")
                .arg(&*output_template_str)
                .arg("--newline")
                .arg("--progress")
                .arg(&url)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            match child {
                Ok(mut child) => {
                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();

                    let (err_tx, ___) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().map_while(Result::ok) {
                            let _ = err_tx.send(line);
                        }
                    });

                    // Process stdout
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        // Parse percentage: [download]  23.5%
                        if line.contains("[download]") && line.contains("%") {
                            if let Some(pct_idx) = line.find('%') {
                                let slice = &line[..pct_idx];
                                if let Some(last_space) = slice.rfind(' ') {
                                    if let Ok(pct) = slice[last_space + 1..].parse::<f32>() {
                                        let _ = tx.send(DownloadEvent::Progress(pct));
                                    }
                                }
                            }
                        }
                    }

                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                // Identify the downloaded file
                                let fallbacks = ["opus", "m4a", "mp3", "wav", "ogg"];
                                let mut downloaded_path = None;
                                for ext in fallbacks {
                                    let p = sounds_dir.join(format!("{}.{}", safe_name, ext));
                                    if p.exists() {
                                        downloaded_path = Some(p);
                                        break;
                                    }
                                }

                                if let Some(final_path) = downloaded_path {
                                    let _ = tx.send(DownloadEvent::Success(
                                        name,
                                        category,
                                        final_path.to_string_lossy().into_owned(),
                                        icon,
                                        url,
                                    ));
                                } else {
                                    let _ = tx.send(DownloadEvent::Error(
                                        "Download success but file not found.".to_string(),
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(DownloadEvent::Error(format!(
                                "Failed to wait on child: {}",
                                e
                            )));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Error(format!(
                        "Failed to start yt-dlp: {}",
                        e
                    )));
                }
            }
        });
    }
}
