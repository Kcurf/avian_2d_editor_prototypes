use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy_egui::egui::{self, ColorImage, TextureHandle, TextureOptions, Vec2 as EguiVec2};
use std::collections::HashMap;
use std::sync::LazyLock;

/// 图片预览缓存管理器，类似 bevy-inspector-egui 的 ScaledDownTextures
#[derive(Default)]
struct ImagePreviewCache {
    /// 缓存的纹理句柄
    textures: HashMap<Handle<Image>, TextureHandle>,
    /// 已处理的图片ID集合
    processed_images: HashMap<Handle<Image>, bool>,
}

static PREVIEW_CACHE: LazyLock<std::sync::Mutex<ImagePreviewCache>> =
    LazyLock::new(|| std::sync::Mutex::new(ImagePreviewCache::default()));

/// 创建图片预览的通用函数
pub fn create_image_preview(
    ctx: &egui::Context,
    images: &Assets<Image>,
    image_handle: &Handle<Image>,
    size: (u32, u32),
) -> Option<TextureHandle> {
    let mut cache = PREVIEW_CACHE.lock().unwrap();

    // 检查缓存
    if let Some(texture_handle) = cache.textures.get(image_handle) {
        return Some(texture_handle.clone());
    }

    // 获取图片数据
    let image = images.get(image_handle)?;

    // 创建缩略图
    let thumbnail_image = create_thumbnail(image, size)?;

    // 创建 egui 纹理
    let texture_handle = ctx.load_texture(
        &format!("preview_{}", image_handle.id()),
        thumbnail_image,
        TextureOptions::default(),
    );

    // 缓存结果
    cache
        .textures
        .insert(image_handle.clone(), texture_handle.clone());
    cache.processed_images.insert(image_handle.clone(), true);

    Some(texture_handle)
}

/// 创建缩略图，类似 bevy-inspector-egui 的 rescaled_image 函数
fn create_thumbnail(image: &Image, target_size: (u32, u32)) -> Option<ColorImage> {
    let width = image.width();
    let height = image.height();

    // 如果图片已经很小，直接使用原尺寸
    if width <= target_size.0 && height <= target_size.1 {
        return create_color_image_from_bevy_image(image);
    }

    // 计算缩放比例
    let scale_x = target_size.0 as f32 / width as f32;
    let scale_y = target_size.1 as f32 / height as f32;
    let scale = scale_x.min(scale_y);

    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    // 简单的缩放实现 - 采样原图片数据
    create_scaled_color_image(image, new_width, new_height)
}

/// 从 Bevy Image 创建 ColorImage
fn create_color_image_from_bevy_image(image: &Image) -> Option<ColorImage> {
    let data = image.data.as_ref()?;
    let width = image.width() as usize;
    let height = image.height() as usize;

    // Bevy Image 数据格式处理
    let pixels: Vec<[u8; 4]> = match image.texture_descriptor.format {
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => {
            // 直接 RGBA 格式
            data.chunks_exact(4)
                .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
                .collect()
        }
        TextureFormat::Rgba32Float => {
            // RGBA32F 转换为 RGBA8
            data.chunks_exact(16)
                .map(|chunk| {
                    let r = (f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) * 255.0)
                        as u8;
                    let g = (f32::from_ne_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) * 255.0)
                        as u8;
                    let b = (f32::from_ne_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]) * 255.0)
                        as u8;
                    let a = (f32::from_ne_bytes([chunk[12], chunk[13], chunk[14], chunk[15]])
                        * 255.0) as u8;
                    [r, g, b, a]
                })
                .collect()
        }
        _ => {
            // 不支持的格式，返回 None
            return None;
        }
    };

    {
        let flat_data: Vec<u8> = pixels.iter().flatten().copied().collect();
        Some(ColorImage::from_rgba_unmultiplied(
            [width, height],
            &flat_data,
        ))
    }
}

/// 创建缩放的 ColorImage（简单的最近邻采样）
fn create_scaled_color_image(image: &Image, new_width: u32, new_height: u32) -> Option<ColorImage> {
    let data = image.data.as_ref()?;
    let src_width = image.width() as usize;
    let src_height = image.height() as usize;
    let dst_width = new_width as usize;
    let dst_height = new_height as usize;

    let mut pixels = vec![[0u8; 4]; dst_width * dst_height];

    // 计算缩放比例
    let scale_x = src_width as f32 / dst_width as f32;
    let scale_y = src_height as f32 / dst_height as f32;

    // 简单的最近邻采样
    for y in 0..dst_height {
        for x in 0..dst_width {
            let src_x = (x as f32 * scale_x) as usize;
            let src_y = (y as f32 * scale_y) as usize;
            let src_index = (src_y * src_width + src_x) * 4;
            let _dst_index = (y * dst_width + x) * 4;

            if src_index + 3 < data.len() {
                pixels[y * dst_width + x] = [
                    data[src_index],
                    data[src_index + 1],
                    data[src_index + 2],
                    data[src_index + 3],
                ];
            }
        }
    }

    {
        let flat_data: Vec<u8> = pixels.iter().flatten().copied().collect();
        Some(ColorImage::from_rgba_unmultiplied(
            [dst_width, dst_height],
            &flat_data,
        ))
    }
}

/// 显示图片预览带悬停信息
pub fn show_image_preview_with_info(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    images: &Assets<Image>,
    image_asset: &crate::ui::asset_management::ImageAssetInfo,
    thumbnail_size: EguiVec2,
) -> bool {
    // 先创建预览纹理
    let texture_handle = create_image_preview(
        ctx,
        images,
        &image_asset.handle,
        (thumbnail_size.x as u32, thumbnail_size.y as u32),
    );

    let response = ui.horizontal(|ui| {
        // 显示图片预览
        if let Some(texture_handle) = &texture_handle {
            ui.add(egui::Image::new(texture_handle).fit_to_exact_size(thumbnail_size));
        } else {
            ui.centered_and_justified(|ui| {
                ui.colored_label(egui::Color32::from_gray(128), "Loading...");
            });
        }

        // 图片信息
        ui.vertical(|ui| {
            ui.label(&image_asset.file_name);
            ui.label(format!("{}×{}", image_asset.size.x, image_asset.size.y));
        });
    });

    // 悬停信息
    response.response.on_hover_ui(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let hover_thumbnail_size = EguiVec2::new(64.0, 64.0);
                if let Some(texture_handle) = &texture_handle {
                    ui.add(
                        egui::Image::new(texture_handle).fit_to_exact_size(hover_thumbnail_size),
                    );
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.colored_label(egui::Color32::from_gray(128), "Loading...");
                    });
                }

                ui.vertical(|ui| {
                    ui.label(&image_asset.file_name);
                    ui.label(format!(
                        "Dimensions: {} x {}",
                        image_asset.size.x, image_asset.size.y
                    ));
                    if let Ok(time) = image_asset.loaded_at.duration_since(std::time::UNIX_EPOCH) {
                        ui.label(format!("Loaded: {}s ago", time.as_secs()));
                    }
                });
            });
        });
    });

    // 返回是否成功显示预览
    texture_handle.is_some()
}
