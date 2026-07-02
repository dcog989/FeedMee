use crate::AppState;
use base64::Engine;
use image::GenericImageView;
use image::imageops::FilterType;
use log::info;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri::State;
use webp::Encoder;

use super::scraper::{compute_content_hash, scrape_og_image};

fn thumbnail_cache_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("thumbnails");
    fs::create_dir_all(&dir).ok();
    Ok(dir)
}

fn hash_url(url: &str) -> String {
    compute_content_hash(url)
}

#[tauri::command]
pub async fn get_thumbnail(
    url: String,
    image_url: String,
    size: u32,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let size = size.clamp(16, 256);
    let resolved = if !image_url.is_empty() {
        image_url
    } else {
        scrape_og_image(&state.http_client, &url)
            .await
            .ok_or_else(|| "No og:image found".to_string())?
    };

    let cache_dir = thumbnail_cache_dir(&app)?;
    let hash = hash_url(&resolved);
    let cache_path = cache_dir.join(format!("{}_{}.webp", hash, size));

    if cache_path.exists() {
        let bytes = fs::read(&cache_path).map_err(|e| e.to_string())?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        return Ok(format!("data:image/webp;base64,{}", encoded));
    }

    let response = state
        .http_client
        .get(&resolved)
        .send()
        .await
        .map_err(|e| format!("Failed to download thumbnail: {}", e))?;

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    if bytes.len() > 50_000_000 {
        return Err("Image too large (>50MB)".to_string());
    }

    let img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(_) => return Err("Failed to decode image".to_string()),
    };

    let (w, h) = img.dimensions();

    let size_f = size as f64;
    let (nw, nh) = if w > h {
        (size, (size_f * h as f64 / w as f64).round() as u32)
    } else {
        ((size_f * w as f64 / h as f64).round() as u32, size)
    };

    let resized =
        image::imageops::resize(&img.to_rgba8(), nw.max(1), nh.max(1), FilterType::Lanczos3);

    let mut canvas = image::RgbaImage::new(size, size);
    let x = (size - nw) / 2;
    let y = (size - nh) / 2;
    image::imageops::overlay(&mut canvas, &resized, x as i64, y as i64);

    let img = image::DynamicImage::ImageRgba8(canvas);
    let webp = Encoder::from_image(&img)
        .map_err(|e| format!("WebP encoder init failed: {}", e))?
        .encode(50.0);

    fs::write(&cache_path, &*webp).ok();

    let encoded = base64::engine::general_purpose::STANDARD.encode(&*webp);
    Ok(format!("data:image/webp;base64,{}", encoded))
}

pub fn cleanup_thumbnail_cache(app: &tauri::AppHandle, max_age_days: u64) -> Result<usize, String> {
    let cache_dir = thumbnail_cache_dir(app)?;
    let cutoff =
        std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_days * 86400);
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata()
                && metadata.is_file()
                && let Ok(modified) = metadata.modified()
                && modified < cutoff
            {
                let _ = fs::remove_file(entry.path());
                count += 1;
            }
        }
    }
    if count > 0 {
        info!(
            "Cleaned up {} stale thumbnail files (max age: {} days)",
            count, max_age_days
        );
    }
    Ok(count)
}
