use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use tauri::AppHandle;
use crate::settings::{get_settings, write_settings, TranscribeAcceleratorSetting};
use crate::engine_supervisor::openvino;

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct NpuStatus {
    pub is_available: bool,
    pub is_enabled: bool,
    pub device_name: String,
    pub model_found: bool,
    pub model_path: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn get_npu_status(app: AppHandle) -> Result<NpuStatus, String> {
    let settings = get_settings(&app);
    let openvino_dirs = openvino::library_dirs();
    
    // Check if OpenVINO C libs exist
    let libs_available = !openvino_dirs.is_empty() 
        || std::env::var("INTEL_OPENVINO_DIR").is_ok()
        || std::env::var("HOME").map(|h| PathBuf::from(h).join("ai/ovgenai/rt").is_dir()).unwrap_or(false);
    
    // Check if model exists
    let ov_models_dir = std::env::var("HANDY_OPENVINO_MODELS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/lib/ollama-models/asr/models/ov"));
    
    let turbo_model = ov_models_dir.join("whisper-large-v3-turbo-int8-ov");
    let model_found = turbo_model.join("openvino_encoder_model.xml").is_file();
    let model_path = if model_found {
        Some(turbo_model.to_string_lossy().to_string())
    } else {
        None
    };

    let is_enabled = settings.transcribe_accelerator == TranscribeAcceleratorSetting::Gpu
        && settings.transcribe_gpu_device.as_deref() == Some("[\"openvino\",\"id\",\"openvino:NPU\"]");

    Ok(NpuStatus {
        is_available: libs_available,
        is_enabled,
        device_name: "Intel NPU 3720 (OpenVINO GenAI)".to_string(),
        model_found,
        model_path,
    })
}

#[tauri::command]
#[specta::specta]
pub fn set_npu_enabled(app: AppHandle, enabled: bool) -> Result<NpuStatus, String> {
    let current_status = get_npu_status(app.clone())?;

    if enabled {
        if !current_status.is_available {
            return Err("OpenVINO GenAI runtime libraries were not detected on this system".to_string());
        }
        if !current_status.model_found {
            return Err("Intel NPU Whisper model (whisper-large-v3-turbo-int8-ov) not found".to_string());
        }
    }

    let mut settings = get_settings(&app);
    if enabled {
        settings.transcribe_accelerator = TranscribeAcceleratorSetting::Gpu;
        settings.transcribe_gpu_device = Some("[\"openvino\",\"id\",\"openvino:NPU\"]".to_string());
    } else {
        settings.transcribe_accelerator = TranscribeAcceleratorSetting::Auto;
        settings.transcribe_gpu_device = None;
    }

    write_settings(&app, settings);
    get_npu_status(app)
}
