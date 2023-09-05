use super::audio::EncodedAudio;
use anyhow::Result;
use reqwest::Url;

pub struct VoiceVoxClient {
    client: reqwest::Client,
    api_base: String,
}

impl VoiceVoxClient {
    pub fn new(api_base: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base,
        }
    }

    pub async fn generate_query(&self, params: GenerateQuery) -> Result<String> {
        let url = Url::parse_with_params(
            &self.get_endpoint("/audio_query"),
            &[("text", params.text), ("speaker", "1".to_string())],
        )?;

        let resp = self.client.post(url).send().await?.text().await?;

        Ok(resp)
    }

    pub async fn synthesis(&self, params: SynthesisParamas) -> Result<EncodedAudio> {
        let url = Url::parse_with_params(
            &self.get_endpoint("/synthesis"),
            &[("speaker", params.style_id.to_string())],
        )?;

        let resp = self
            .client
            .post(url)
            .body(params.query)
            .send()
            .await?
            .bytes()
            .await?;

        Ok(EncodedAudio(resp.to_vec()))
    }

    pub async fn initialize_speaker(&self, speaker_id: i64) -> Result<()> {
        let url = Url::parse_with_params(
            &self.get_endpoint("/initialize_speaker"),
            &[("speaker", speaker_id.to_string())],
        )?;

        self.client.post(url).send().await?;

        Ok(())
    }

    fn get_endpoint(&self, path: impl AsRef<str>) -> String {
        self.api_base.clone() + path.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct GenerateQuery {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct SynthesisParamas {
    pub style_id: i64,
    pub query: String,
}
