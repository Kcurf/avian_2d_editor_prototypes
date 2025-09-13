use bevy::prelude::*;
use bevy_egui::egui::{
    self,
    epaint::text::{FontData, FontInsert, FontPriority, InsertFontFamily},
};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use log::{debug, info, warn};
use std::fs::read;

pub(super) fn initialize_fonts(ctx: &egui::Context) {
    debug!("Loading system fonts and applying styling");
    // Define font mappings for simplified Chinese
    let font_names = [
        "Heiti SC",
        "Songti SC",
        "Noto Sans CJK SC",
        "Noto Sans SC",
        "WenQuanYi Zen Hei",
        "SimSun",
        "PingFang SC",
        "Source Han Sans CN",
    ];

    // Additional fallback fonts
    let fallback_fonts = [
        "Arial",
        "Helvetica",
        "Times New Roman",
        "Georgia",
        "Verdana",
        "Segoe UI",
        "Tahoma",
    ];

    // Load fonts for simplified Chinese
    if let Some(font_data) = load_font_family(&font_names) {
        info!("Loaded font data for simplified Chinese");
        ctx.add_font(FontInsert::new(
            "Chinese",
            FontData::from_owned(font_data),
            vec![InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: FontPriority::Highest,
            }],
        ));
    } else {
        warn!(
            "Could not load font data for simplified Chinese. Available fonts: {}",
            font_names.join(", ")
        );
    }

    // Also load fallback fonts
    for &fallback_name in &fallback_fonts {
        if let Some(font_data) = load_font_family(&[fallback_name]) {
            debug!("Loaded fallback font: {}", fallback_name);
            ctx.add_font(FontInsert::new(
                fallback_name,
                FontData::from_owned(font_data),
                vec![InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: FontPriority::Lowest,
                }],
            ));
        }
    }

    info!("System font loading completed");
}

/// Attempt to load a system font by any of the given family names
fn load_font_family(family_names: &[&str]) -> Option<Vec<u8>> {
    let system_source = SystemSource::new();

    for &name in family_names {
        let font_handle = system_source
            .select_best_match(&[FamilyName::Title(name.to_string())], &Properties::new());

        match font_handle {
            Ok(handle) => match handle {
                Handle::Memory { bytes, .. } => {
                    debug!("Loaded font '{}' from memory", name);
                    return Some(bytes.to_vec());
                }
                Handle::Path { path, .. } => {
                    debug!("Loaded font '{}' from path: {:?}", name, path);
                    if let Ok(data) = read(path) {
                        return Some(data);
                    }
                }
            },
            Err(e) => {
                debug!("Could not load font '{}': {:?}", name, e);
            }
        }
    }

    None
}
