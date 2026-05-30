use crate::AppState;
use base64::Engine;
use image::ExtendedColorType;
use image::GenericImageView;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::BufWriter;
use std::path::PathBuf;
use tauri::Manager;
use tauri::State;

use super::scraper::scrape_og_image;

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
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[tauri::command]
pub async fn get_thumbnail(
    url: String,
    image_url: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let resolved = if !image_url.is_empty() {
        image_url
    } else {
        scrape_og_image(&state.http_client, &url)
            .await
            .ok_or_else(|| "No og:image found".to_string())?
    };

    let cache_dir = thumbnail_cache_dir(&app)?;
    let hash = hash_url(&resolved);
    let cache_path = cache_dir.join(format!("{}.webp", hash));

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

    if bytes.len() > 2_000_000 {
        return Err("Image too large (>500KB)".to_string());
    }

    let img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(_) => return Err("Failed to decode image".to_string()),
    };

    let (w, h) = img.dimensions();

    let (nw, nh) = if w > h {
        (56u32, (56_f64 * h as f64 / w as f64).round() as u32)
    } else {
        ((56_f64 * w as f64 / h as f64).round() as u32, 56u32)
    };

    let resized =
        image::imageops::resize(&img.to_rgba8(), nw.max(1), nh.max(1), FilterType::Lanczos3);

    let mut canvas = image::RgbaImage::new(56, 56);
    let x = (56 - nw) / 2;
    let y = (56 - nh) / 2;
    image::imageops::overlay(&mut canvas, &resized, x as i64, y as i64);

    let (rw, rh) = (56u32, 56u32);

    let mut webp_buf = Vec::new();
    {
        let encoder = WebPEncoder::new_lossless(BufWriter::new(&mut webp_buf));
        encoder
            .encode(canvas.as_raw(), rw, rh, ExtendedColorType::Rgba8)
            .map_err(|e| format!("WebP encoding failed: {}", e))?;
    }

    fs::write(&cache_path, &webp_buf).ok();

    let encoded = base64::engine::general_purpose::STANDARD.encode(&webp_buf);
    Ok(format!("data:image/webp;base64,{}", encoded))
}
