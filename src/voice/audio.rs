use crate::voice::ffmpeg::convert_pcm_s16le;
use anyhow::Result;

pub struct EncodedAudio(pub Vec<u8>);

impl EncodedAudio {
    pub async fn decode(self) -> Result<DecodedAudio> {
        let decoded_buf = convert_pcm_s16le(self.0).await?;
        Ok(DecodedAudio(decoded_buf))
    }
}

impl From<Vec<u8>> for EncodedAudio {
    fn from(buf: Vec<u8>) -> Self {
        Self(buf)
    }
}

impl From<EncodedAudio> for Vec<u8> {
    fn from(audio: EncodedAudio) -> Self {
        audio.0
    }
}

pub struct DecodedAudio(Vec<u8>);

impl From<Vec<u8>> for DecodedAudio {
    fn from(buf: Vec<u8>) -> Self {
        Self(buf)
    }
}

impl From<DecodedAudio> for Vec<u8> {
    fn from(audio: DecodedAudio) -> Self {
        audio.0
    }
}
