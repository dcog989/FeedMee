use crate::AppState;
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader, Limits};
use log::info;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tauri::State;
use webp::Encoder;

use super::scraper::{scrape_og_image, short_hash};

const MAX_IMAGE_BYTES: usize = 10_000_000;
/// A feed image bigger than this is not a thumbnail source; refuse it before
/// decoding, so a compressed image cannot expand into gigabytes of pixels.
const MAX_IMAGE_DIMENSION: u32 = 8192;
const MAX_IMAGE_ALLOC_BYTES: u64 = 64 * 1024 * 1024;
const THUMBNAIL_QUALITY: f32 = 50.0;

fn process_thumbnail(bytes: &[u8], size: u32) -> Result<Vec<u8>, String> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC_BYTES);

    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "Failed to read image format".to_string())?;
    reader.limits(limits);
    let img = reader.decode().map_err(|_| "Failed to decode image".to_string())?;

    let (w, h) = img.dimensions();

    let size_f = size as f64;
    let (nw, nh) = if w > h {
        (size, (size_f * h as f64 / w as f64).round() as u32)
    } else {
        ((size_f * w as f64 / h as f64).round() as u32, size)
    };

    let resized = image::imageops::resize(&img.to_rgba8(), nw.max(1), nh.max(1), FilterType::Triangle);

    let mut canvas = image::RgbaImage::new(size, size);
    let x = (size - nw) / 2;
    let y = (size - nh) / 2;
    image::imageops::overlay(&mut canvas, &resized, x as i64, y as i64);

    let img = image::DynamicImage::ImageRgba8(canvas);
    let webp = Encoder::from_image(&img)
        .map_err(|e| format!("WebP encoder init failed: {}", e))?
        .encode(THUMBNAIL_QUALITY);

    Ok(webp.to_vec())
}

fn write_thumbnail(path: &Path, bytes: &[u8]) -> Result<(), String> {
    // Write then rename, so a crash cannot leave a truncated file in the cache.
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)
        .and_then(|()| {
            let _ = fs::remove_file(path);
            fs::rename(&tmp, path)
        })
        .map_err(|e| format!("Failed to cache thumbnail: {}", e))
}

fn path_string(path: PathBuf) -> Result<String, String> {
    path.into_os_string()
        .into_string()
        .map_err(|_| "Thumbnail path is not valid UTF-8".to_string())
}

async fn touch_thumbnail(path: PathBuf) {
    let _ = tauri::async_runtime::spawn_blocking(move || {
        if let Ok(file) = fs::File::options().write(true).open(&path) {
            let now = std::time::SystemTime::now();
            let _ = file.set_times(fs::FileTimes::new().set_modified(now));
        }
    })
    .await;
}

#[tauri::command]
pub async fn get_thumbnail(
    url: String,
    image_url: String,
    size: u32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let size = size.clamp(16, 256);
    let client = state.http_client.clone();

    let cache_dir = state.paths.thumbnail_dir();
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create thumbnail cache: {}", e))?;
    let source_key = if image_url.is_empty() {
        url.as_str()
    } else {
        image_url.as_str()
    };
    let cache_path = cache_dir.join(format!("{}_{}.webp", short_hash(source_key), size));

    if cache_path.exists() {
        touch_thumbnail(cache_path.clone()).await;
        return path_string(cache_path);
    }

    let resolved = if image_url.is_empty() {
        scrape_og_image(&client, &url)
            .await
            .ok_or_else(|| "No og:image found".to_string())?
    } else {
        image_url
    };

    if !crate::net::is_public_url(&resolved) {
        return Err("Thumbnail URL is not a public address".to_string());
    }

    let mut response = client
        .get(&resolved)
        .send()
        .await
        .map_err(|e| format!("Failed to download thumbnail: {}", e))?;

    if response
        .content_length()
        .is_some_and(|len| len > MAX_IMAGE_BYTES as u64)
    {
        return Err(format!("Image too large (>{}MB)", MAX_IMAGE_BYTES / 1_000_000));
    }

    let mut bytes: Vec<u8> = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > MAX_IMAGE_BYTES {
            return Err(format!("Image too large (>{}MB)", MAX_IMAGE_BYTES / 1_000_000));
        }
        bytes.extend_from_slice(&chunk);
    }

    let write_path = cache_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let webp = process_thumbnail(&bytes, size)?;
        write_thumbnail(&write_path, &webp)
    })
    .await
    .map_err(|e| e.to_string())??;

    path_string(cache_path)
}

/// Delete thumbnails older than `max_age_days`, then the oldest ones until the
/// cache is under `max_bytes`. Returns (files, bytes) removed.
pub fn cleanup_thumbnail_cache(cache_dir: &Path, max_age_days: u64, max_bytes: u64) -> (usize, u64) {
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_days * 86_400);
    let mut remaining: Vec<(PathBuf, std::time::SystemTime, u64)> = Vec::new();
    let mut removed_files = 0;
    let mut removed_bytes = 0;

    if let Ok(entries) = fs::read_dir(cache_dir) {
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            if modified < cutoff {
                if fs::remove_file(entry.path()).is_ok() {
                    removed_files += 1;
                    removed_bytes += metadata.len();
                }
            } else {
                remaining.push((entry.path(), modified, metadata.len()));
            }
        }
    }

    let mut total: u64 = remaining.iter().map(|(_, _, len)| len).sum();
    remaining.sort_by_key(|(_, modified, _)| *modified);
    for (path, _, len) in remaining {
        if total <= max_bytes {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(len);
            removed_files += 1;
            removed_bytes += len;
        }
    }

    if removed_files > 0 {
        info!(
            "Cleaned up {} stale thumbnail files ({} bytes)",
            removed_files, removed_bytes
        );
    }
    (removed_files, removed_bytes)
}

#[cfg(test)]
mod tests {
    use super::process_thumbnail;

    #[test]
    fn rejects_undecodable_bytes() {
        assert!(process_thumbnail(b"not an image", 64).is_err());
    }
}
