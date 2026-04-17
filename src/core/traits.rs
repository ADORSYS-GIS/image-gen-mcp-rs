use async_trait::async_trait;

use crate::core::errors::DomainResult;

/// Parameters for image generation, flavor-specific
#[derive(Debug, Clone)]
pub struct GenerateParams {
    pub prompt: String,
    pub model: Option<String>,
    pub n: Option<u8>,
    pub size: Option<String>,
    pub ratio: Option<String>,
    pub quality: Option<String>,
    pub style: Option<String>,
    pub seed: Option<i64>,
}

impl GenerateParams {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            n: None,
            size: None,
            ratio: None,
            quality: None,
            style: None,
            seed: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn with_ratio(mut self, ratio: impl Into<String>) -> Self {
        self.ratio = Some(ratio.into());
        self
    }

    pub fn with_n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }

    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }
}

/// Port for generating images
#[async_trait]
pub trait ImageGenerationPort: Send + Sync {
    async fn generate(&self, params: GenerateParams) -> DomainResult<Vec<String>>;
}
