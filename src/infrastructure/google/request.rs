use super::types::*;

pub struct GeminiRequestBuilder;

impl GeminiRequestBuilder {
    pub fn build(
        prompt: String,
        image_data: Option<(String, String)>,
        seed: Option<i64>,
    ) -> GeminiRequest {
        let mut parts = Vec::new();

        parts.push(GeminiPart::Text(prompt));

        if let Some((mime_type, data)) = image_data {
            parts.push(GeminiPart::InlineData(InlineData { mime_type, data }));
        }

        GeminiRequest {
            contents: vec![GeminiContent { parts }],
            generation_config: GenerationConfig {
                response_modalities: vec!["IMAGE".to_string()],
                seed,
            },
        }
    }
}
