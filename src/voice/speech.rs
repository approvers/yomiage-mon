use super::voicevox::{GenerateQuery, VoiceVoxClient};
use super::{audio::EncodedAudio, voicevox::SynthesisParamas};
use anyhow::Result;

pub async fn initialize_speakers(client: &VoiceVoxClient) -> Result<()> {
    client.initialize_speaker(1).await?;

    Ok(())
}

pub async fn make_speech(client: &VoiceVoxClient, option: SpeechRequest) -> Result<EncodedAudio> {
    let query = client
        .generate_query(GenerateQuery { text: option.text })
        .await?;

    let audio = client
        .synthesis(SynthesisParamas { style_id: 1, query })
        .await?;

    Ok(audio)
}

pub struct SpeechRequest {
    pub text: String,
}
