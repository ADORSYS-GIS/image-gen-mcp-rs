use async_openai::types::images::{ImageModel, ImageSize, ImageQuality, ImageStyle};
use crate::core::config::AppConfig;

pub struct OpenAiUtils;

impl OpenAiUtils {
    pub fn parse_size(size: Option<&str>) -> ImageSize {
        match size {
            Some("256") => ImageSize::S256x256,
            Some("512") => ImageSize::S512x512,
            Some("1024") => ImageSize::S1024x1024,
            Some("1792x1024") => ImageSize::S1792x1024,
            Some("1024x1792") => ImageSize::S1024x1792,
            Some("1536x1024") => ImageSize::S1536x1024,
            Some("1024x1536") => ImageSize::S1024x1536,
            _ => ImageSize::S1024x1024,
        }
    }

    pub fn parse_model(model: Option<&str>, config: &AppConfig) -> ImageModel {
        let model_name = match model {
            Some("dall-e") => "dall-e-3",
            Some("gpt-image-1") => "gpt-image-1",
            Some(m) => m,
            None => &config.image_model,
        };
        ImageModel::Other(model_name.to_string())
    }

    pub fn parse_quality(quality: Option<&str>) -> Option<ImageQuality> {
        match quality {
            Some("hd") => Some(ImageQuality::HD),
            Some("standard") => Some(ImageQuality::Standard),
            _ => None,
        }
    }

    pub fn parse_style(style: Option<&str>) -> Option<ImageStyle> {
        match style {
            Some("vivid") => Some(ImageStyle::Vivid),
            Some("natural") => Some(ImageStyle::Natural),
            _ => None,
        }
    }
}
