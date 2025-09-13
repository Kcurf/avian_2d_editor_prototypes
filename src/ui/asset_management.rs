use crate::tr;
use bevy::{
    asset::{Handle, RenderAssetUsages},
    image::{CompressedImageFormats, ImageFormat, ImageSampler},
    prelude::*,
    tasks::AsyncComputeTaskPool,
    time::common_conditions::on_timer,
};
use crossbeam::channel::{Receiver, Sender, bounded};
use rfd::AsyncFileDialog;
use std::{borrow::Cow, time::Duration};

/// 选择的图片资产状态资源
#[derive(Resource, Default)]
pub struct SelectedImageAsset {
    /// 当前选择的图片句柄
    pub handle: Option<Handle<Image>>,
    /// 当前选择的图片显示名称
    pub display_name: String,
}

/// 资产管理插件
pub struct AssetManagementPlugin;

impl Plugin for AssetManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ImageAssetChannel>()
            .init_resource::<SelectedImageAsset>()
            .add_systems(
                Update,
                image_asset_watcher.run_if(on_timer(Duration::from_millis(50))),
            );
    }
}

/// 图片资产通道
#[derive(Resource)]
pub struct ImageAssetChannel {
    /// 最后加载的文件名
    pub last_file_name: String,
    /// 发送端
    pub send: Sender<ImageAssetWrapper>,
    /// 接收端
    rec: Receiver<ImageAssetWrapper>,
    /// 已加载的图片资产列表
    pub loaded_assets: Vec<ImageAssetInfo>,
}

/// 图片资产包装器
pub struct ImageAssetWrapper {
    /// 文件名
    file_name: String,
    /// 图片数据
    image: Image,
}

/// 图片资产信息
#[derive(Clone, Debug)]
pub struct ImageAssetInfo {
    /// 文件名
    pub file_name: String,
    /// 图片句柄
    pub handle: Handle<Image>,
    /// 图片尺寸
    pub size: Vec2,
    /// 加载时间
    pub loaded_at: std::time::SystemTime,
}

impl Default for ImageAssetChannel {
    fn default() -> Self {
        let (tx, rx) = bounded(1);
        Self {
            last_file_name: tr!("no_asset_selected"),
            send: tx,
            rec: rx,
            loaded_assets: Vec::new(),
        }
    }
}

/// 图片资产监听器
fn image_asset_watcher(
    mut image_channel: ResMut<ImageAssetChannel>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(asset_wrapper) = image_channel.rec.try_recv() else {
        return;
    };

    // 更新最后文件名
    image_channel.last_file_name = asset_wrapper.file_name.clone();

    // 生成句柄
    let handle = Handle::Weak(AssetId::Uuid {
        uuid: uuid::Uuid::new_v4(),
    });

    // 插入图片到资源管理器
    let image_size = Vec2::new(
        asset_wrapper.image.width() as f32,
        asset_wrapper.image.height() as f32,
    );
    images.insert(&handle, asset_wrapper.image);

    // 添加到已加载资产列表
    let file_name = asset_wrapper.file_name.clone();
    let asset_info = ImageAssetInfo {
        file_name: file_name.clone(),
        handle: handle.clone(),
        size: image_size,
        loaded_at: std::time::SystemTime::now(),
    };

    // 检查是否已存在相同文件名的资产
    if let Some(pos) = image_channel
        .loaded_assets
        .iter()
        .position(|asset| asset.file_name == asset_info.file_name)
    {
        image_channel.loaded_assets[pos] = asset_info;
    } else {
        image_channel.loaded_assets.push(asset_info);
    }

    info!("Image asset loaded: {}", file_name);
}

/// 打开图片加载对话框
pub fn open_load_image_dialog(
    sender: Sender<ImageAssetWrapper>,
    supported_extensions: Vec<Cow<'static, str>>,
) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            if let Some(handles) = AsyncFileDialog::new()
                .set_title(tr!("select_image_asset"))
                .add_filter(tr!("image"), &supported_extensions)
                .pick_files()
                .await
            {
                for handle in handles {
                    let bytes = handle.read().await;

                    // 根据文件扩展名确定图片格式
                    let file_extension = handle
                        .file_name()
                        .rsplit('.')
                        .next()
                        .unwrap_or("")
                        .to_lowercase();

                    // 首先尝试使用正确的格式
                    let image_result = match file_extension.as_str() {
                        "png" => Image::from_buffer(
                            &bytes,
                            bevy::image::ImageType::Format(ImageFormat::Png),
                            CompressedImageFormats::NONE,
                            false,
                            ImageSampler::nearest(),
                            RenderAssetUsages::RENDER_WORLD,
                        ),
                        "jpg" | "jpeg" => Image::from_buffer(
                            &bytes,
                            bevy::image::ImageType::Format(ImageFormat::Jpeg),
                            CompressedImageFormats::NONE,
                            false,
                            ImageSampler::nearest(),
                            RenderAssetUsages::RENDER_WORLD,
                        ),
                        "bmp" => Image::from_buffer(
                            &bytes,
                            bevy::image::ImageType::Format(ImageFormat::Bmp),
                            CompressedImageFormats::NONE,
                            false,
                            ImageSampler::nearest(),
                            RenderAssetUsages::RENDER_WORLD,
                        ),
                        "tga" => Image::from_buffer(
                            &bytes,
                            bevy::image::ImageType::Format(ImageFormat::Tga),
                            CompressedImageFormats::NONE,
                            false,
                            ImageSampler::nearest(),
                            RenderAssetUsages::RENDER_WORLD,
                        ),
                        "webp" => Image::from_buffer(
                            &bytes,
                            bevy::image::ImageType::Format(ImageFormat::WebP),
                            CompressedImageFormats::NONE,
                            false,
                            ImageSampler::nearest(),
                            RenderAssetUsages::RENDER_WORLD,
                        ),
                        // 对于未知格式，尝试自动检测
                        _ => Image::from_buffer(
                            &bytes,
                            bevy::image::ImageType::Extension(&file_extension),
                            CompressedImageFormats::NONE,
                            false,
                            ImageSampler::nearest(),
                            RenderAssetUsages::RENDER_WORLD,
                        ),
                    };

                    let image = match image_result {
                        Ok(img) => img,
                        Err(err) => {
                            error!("Failed to load image!\n\n {:?}", err);
                            return;
                        }
                    };

                    let asset_wrapper = ImageAssetWrapper {
                        image,
                        file_name: handle.file_name(),
                    };

                    match sender.send(asset_wrapper) {
                        Ok(_) => (),
                        Err(err) => {
                            error!("Channel failed!\n\n {:?}", err);
                        }
                    };
                }
            }
        })
        .detach();
}

/// 获取当前加载的图片资产
pub fn get_available_images(image_channel: &ImageAssetChannel) -> Vec<ImageAssetInfo> {
    image_channel.loaded_assets.clone()
}

/// 获取支持的图片扩展名列表
pub fn get_supported_image_extensions() -> Vec<Cow<'static, str>> {
    vec![
        "png".into(),
        "jpg".into(),
        "jpeg".into(),
        "bmp".into(),
        "tga".into(),
        "webp".into(),
    ]
}
