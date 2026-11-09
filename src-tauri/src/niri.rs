use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct NiriStatus {
    pub is_niri: bool,
    pub is_configured: bool,
    pub config_path: Option<String>,
    pub shortcut: Option<String>,
}

pub fn is_niri_session() -> bool {
    if let Ok(socket) = std::env::var("NIRI_SOCKET") {
        if Path::new(&socket).exists() {
            return true;
        }
    }
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| v.to_lowercase().contains("niri"))
        .unwrap_or(false)
}

pub fn get_niri_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let config_dir = PathBuf::from(home).join(".config").join("niri");
    let my_binds = config_dir.join("my-binds.kdl");
    if my_binds.is_file() {
        return Some(my_binds);
    }
    let config = config_dir.join("config.kdl");
    if config.is_file() {
        return Some(config);
    }
    None
}

fn sanitize_shortcut_key(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Shortcut key cannot be empty".to_string());
    }
    // Allow standard Niri modifiers and keys (e.g. "Ctrl+Space", "Mod+Space", "Alt+Shift+F")
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '_' || c == '-') {
        return Err(format!("Invalid characters in shortcut key: {trimmed}"));
    }
    Ok(trimmed.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn check_niri_status() -> NiriStatus {
    let is_niri = is_niri_session();
    if !is_niri {
        return NiriStatus {
            is_niri: false,
            is_configured: false,
            config_path: None,
            shortcut: None,
        };
    }

    let config_path = get_niri_config_path();
    let mut is_configured = false;
    let mut shortcut = None;

    if let Some(ref path) = config_path {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                // Skip commented lines
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("/-") {
                    continue;
                }
                if trimmed.contains("--toggle-transcription") {
                    is_configured = true;
                    if let Some(key_part) = trimmed.split_whitespace().next() {
                        shortcut = Some(key_part.to_string());
                    }
                    break;
                }
            }
        }
    }

    NiriStatus {
        is_niri: true,
        is_configured,
        config_path: config_path.map(|p| p.to_string_lossy().to_string()),
        shortcut,
    }
}

#[tauri::command]
#[specta::specta]
pub fn install_niri_shortcut(shortcut_key: Option<String>) -> Result<NiriStatus, String> {
    if !is_niri_session() {
        return Err("Not running in an active Niri Wayland session".to_string());
    }

    let key = match shortcut_key {
        Some(k) => sanitize_shortcut_key(&k)?,
        None => "Ctrl+Space".to_string(),
    };

    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let config_dir = PathBuf::from(home).join(".config").join("niri");
    fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create niri directory: {e}"))?;

    let target_file = config_dir.join("my-binds.kdl");
    let main_config = config_dir.join("config.kdl");

    // Ensure config.kdl includes my-binds.kdl if config.kdl exists
    if main_config.is_file() {
        if let Ok(main_content) = fs::read_to_string(&main_config) {
            if !main_content.contains("my-binds.kdl") {
                let mut f = fs::OpenOptions::new()
                    .append(true)
                    .open(&main_config)
                    .map_err(|e| format!("Failed to append include to config.kdl: {e}"))?;
                writeln!(f, "\ninclude \"my-binds.kdl\"")
                    .map_err(|e| format!("Failed to write include to config.kdl: {e}"))?;
            }
        }
    }

    let bind_line = format!(
        "    {} repeat=false hotkey-overlay-title=\"Диктовка (Handy)\" {{ spawn \"handy\" \"--toggle-transcription\"; }}",
        key
    );

    let content = if target_file.is_file() {
        fs::read_to_string(&target_file).map_err(|e| format!("Failed to read {}: {e}", target_file.display()))?
    } else {
        "binds {\n}\n".to_string()
    };

    let mut replaced = false;
    let mut new_lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//") && trimmed.contains("--toggle-transcription") {
            new_lines.push(bind_line.clone());
            replaced = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !replaced {
        if let Some(last_brace_idx) = new_lines.iter().rposition(|l| l.trim() == "}") {
            new_lines.insert(last_brace_idx, bind_line);
        } else {
            new_lines.push(bind_line);
        }
    }

    let final_content = new_lines.join("\n") + "\n";

    // Atomic write via temporary file in the same filesystem directory
    let temp_file = config_dir.join(format!(".my-binds.kdl.tmp.{}", std::process::id()));
    {
        let mut tmp = File::create(&temp_file)
            .map_err(|e| format!("Failed to create temporary file {}: {e}", temp_file.display()))?;
        tmp.write_all(final_content.as_bytes())
            .map_err(|e| format!("Failed to write temporary config: {e}"))?;
        tmp.sync_all()
            .map_err(|e| format!("Failed to sync temporary config to disk: {e}"))?;
    }

    fs::rename(&temp_file, &target_file)
        .map_err(|e| format!("Failed to atomically replace {}: {e}", target_file.display()))?;

    // Reload Niri config via IPC with status inspection
    match Command::new("niri").args(["msg", "action", "load-config-file"]).output() {
        Ok(output) => {
            if !output.status.success() {
                eprintln!(
                    "[WARN] 'niri msg action load-config-file' returned non-zero exit code: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("[WARN] Failed to invoke niri IPC: {e}");
        }
    }

    Ok(check_niri_status())
}
