use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    pub id: String,
    pub name: String,
    pub category: String,
    pub file_path: String,
    #[serde(default = "default_volume")]
    pub volume_linear: f32,
    #[serde(default = "default_icon")]
    pub icon: String,
    pub url: Option<String>,
    #[serde(skip)]
    pub error_state: bool,
}

fn default_volume() -> f32 {
    0.5
}

fn default_icon() -> String {
    "🎵".to_string()
}

#[derive(Debug, Deserialize)]
struct SoundEntry {
    name: Option<String>,
    file: Option<String>,
    #[serde(default = "default_volume")]
    volume: f32,
    #[serde(default = "default_icon")]
    icon: String,
    url: Option<String>,
}

pub fn get_bundled_sounds() -> Vec<Sound> {
    match load_sounds_from_file("assets/sounds.toml") {
        Ok(sounds) => sounds,
        Err(e) => {
            eprintln!(
                "Warning: Failed to load bundled sounds.toml: {}. Loading empty list.",
                e
            );
            Vec::new()
        }
    }
}

pub fn load_custom_sounds() -> Vec<Sound> {
    let path = if let Some(proj_dirs) = ProjectDirs::from("com", "tanin", "tanin") {
        proj_dirs.config_dir().join("sounds.toml")
    } else {
        PathBuf::from("custom_sounds.toml")
    };

    if !path.exists() {
        return Vec::new();
    }

    match load_sounds_from_file(&path) {
        Ok(sounds) => sounds,
        Err(e) => {
            eprintln!(
                "Warning: Failed to load custom sounds from {:?}: {}",
                path, e
            );
            Vec::new()
        }
    }
}

pub fn load_sounds_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Sound>> {
    let content = fs::read_to_string(&path).context("Could not read sounds configuration file")?;
    let root: toml::Table =
        toml::from_str(&content).context("Could not parse sounds configuration file")?;

    let base_path = root
        .get("base_path")
        .and_then(|v| v.as_str())
        .map(|s| s.trim_end_matches('/').to_string());

    let base_path_str = base_path.unwrap_or_else(|| "assets/sounds".to_string());

    let mut sounds = Vec::new();

    for (category_name, category_value) in &root {
        if category_name == "base_path" {
            continue;
        }

        if let Some(sound_map) = category_value.as_table() {
            for (sound_id, sound_data) in sound_map {
                let entry: SoundEntry = sound_data
                    .clone()
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("Failed to parse sound '{}': {}", sound_id, e))?;

                let name = entry
                    .name
                    .clone()
                    .unwrap_or_else(|| sound_id.replace("_", " "));

                let filename = entry.file.clone().unwrap_or_else(|| {
                    let slug = name.to_lowercase().replace(" ", "_");
                    format!("{}.ogg", slug)
                });

                let file_path = if Path::new(&filename).is_absolute() {
                    filename
                } else {
                    format!("{}/{}", base_path_str, filename)
                };

                sounds.push(Sound {
                    id: sound_id.clone(),
                    name,
                    category: category_name.clone(),
                    file_path,
                    volume_linear: entry.volume,
                    icon: entry.icon,
                    url: entry.url,
                    error_state: false,
                });
            }
        }
    }

    // Sort for consistent order (optional but nice)
    sounds.sort_by(|a, b| {
        let cat_cmp = a.category.cmp(&b.category);
        if cat_cmp == std::cmp::Ordering::Equal {
            a.id.cmp(&b.id)
        } else {
            cat_cmp
        }
    });

    Ok(sounds)
}

pub fn add_custom_sound(
    name: &str,
    category: &str,
    file_path: &str,
    icon: &str,
    url: Option<&str>,
) -> Result<()> {
    let toml_path = if let Some(proj_dirs) = ProjectDirs::from("com", "tanin", "tanin") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }
        config_dir.join("sounds.toml")
    } else {
        PathBuf::from("custom_sounds.toml")
    };

    let mut root: toml::Table = if toml_path.exists() {
        let content = fs::read_to_string(&toml_path)?;
        toml::from_str(&content).unwrap_or_else(|_| toml::Table::new())
    } else {
        toml::Table::new()
    };

    let category_entry = root
        .entry(category)
        .or_insert(toml::Value::Table(toml::Table::new()));

    if let toml::Value::Table(cat_table) = category_entry {
        let id = name.to_lowercase().replace(" ", "_");

        let mut sound_entry = toml::Table::new();
        sound_entry.insert(
            "file".to_string(),
            toml::Value::String(file_path.to_string()),
        );
        sound_entry.insert("icon".to_string(), toml::Value::String(icon.to_string()));
        sound_entry.insert("volume".to_string(), toml::Value::Float(0.5));

        if let Some(u) = url {
            sound_entry.insert("url".to_string(), toml::Value::String(u.to_string()));
        }

        cat_table.insert(id, toml::Value::Table(sound_entry));
    }

    let output = toml::to_string_pretty(&root)?;
    fs::write(toml_path, output)?;

    Ok(())
}
