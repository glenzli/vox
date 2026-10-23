//! Bounded streaming PCM concatenation and explicitly selected format conversion.
use super::{ChapterCompilationOutput, PreparedChapter, frame_ms, rounded_ratio};
use crate::{
    ContentRef,
    audio::decode_pcm_s16le_wav,
    project::{Error, Result},
};
use std::{
    f64::consts::PI,
    fs,
    io::{BufWriter, Write},
    path::Path,
};

struct Sink {
    output: BufWriter<fs::File>,
    digest: blake3::Hasher,
    bytes: u64,
}

impl Sink {
    fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.output.write_all(bytes)?;
        self.digest.update(bytes);
        self.bytes += bytes.len() as u64;
        Ok(())
    }

    fn silence(&mut self, mut bytes: u64) -> Result<()> {
        let zeros = [0_u8; 65_536];
        while bytes > 0 {
            let count = bytes.min(zeros.len() as u64) as usize;
            self.write(&zeros[..count])?;
            bytes -= count as u64;
        }
        Ok(())
    }
}

pub(super) fn write_compilation(
    staging: &Path,
    chapters: &[PreparedChapter],
    output: &ChapterCompilationOutput,
) -> Result<ContentRef> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(staging.join("compilation.wav"))?;
    let mut sink = Sink {
        output: BufWriter::new(file),
        digest: blake3::Hasher::new(),
        bytes: 0,
    };
    let pcm_size =
        u32::try_from(output.byte_length - 44).map_err(|_| Error::new("compilation_too_large"))?;
    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(pcm_size + 36).to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&16_u32.to_le_bytes());
    header.extend_from_slice(&1_u16.to_le_bytes());
    header.extend_from_slice(&output.channels.to_le_bytes());
    header.extend_from_slice(&output.sample_rate_hz.to_le_bytes());
    header
        .extend_from_slice(&(output.sample_rate_hz * u32::from(output.channels) * 2).to_le_bytes());
    header.extend_from_slice(&(output.channels * 2).to_le_bytes());
    header.extend_from_slice(&16_u16.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&pcm_size.to_le_bytes());
    sink.write(&header)?;
    for (index, chapter) in chapters.iter().enumerate() {
        if index > 0 {
            sink.silence(output.gap_frames * u64::from(output.channels) * 2)?;
        }
        let audio = chapter
            .audio
            .as_ref()
            .ok_or_else(|| Error::new("chapter_audio_unaccepted"))?;
        let bytes = chapter.project.object(&audio.content)?;
        let (contract, pcm) = decode_pcm_s16le_wav(&bytes, audio.contract.origin)?;
        if contract != audio.contract {
            return Err(Error::new("corrupt_audio"));
        }
        let frames = rounded_ratio(
            contract.frame_count,
            u64::from(output.sample_rate_hz),
            u64::from(contract.sample_rate_hz),
        )?;
        if Some(frames) != chapter.preview.output_frames {
            return Err(Error::new("compilation_preflight_changed"));
        }
        if (contract.sample_rate_hz, contract.channels) == (output.sample_rate_hz, output.channels)
        {
            sink.write(pcm)?;
        } else {
            if output.sample_rate_hz != 24_000 || output.channels != 1 || contract.channels > 2 {
                return Err(Error::new("compilation_format_mismatch"));
            }
            convert(
                &mut sink,
                pcm,
                contract.sample_rate_hz,
                contract.channels,
                frames,
            )?;
        }
    }
    if sink.bytes != output.byte_length
        || frame_ms(output.frames, output.sample_rate_hz) != output.duration_ms
    {
        return Err(Error::new("compilation_size_mismatch"));
    }
    sink.output.flush()?;
    sink.output.get_ref().sync_all()?;
    ContentRef::new(
        crate::ContentDigest::from_digest_bytes(*sink.digest.finalize().as_bytes()),
        "audio/wav",
        sink.bytes,
    )
    .map_err(|_| Error::new("corrupt_audio"))
}

fn convert(sink: &mut Sink, pcm: &[u8], input_rate: u32, channels: u16, frames: u64) -> Result<()> {
    let channels = usize::from(channels);
    let input_frames = pcm.len() / (channels * 2);
    let sample = |frame: usize| -> f64 {
        let at = frame * channels * 2;
        (0..channels)
            .map(|channel| {
                let at = at + channel * 2;
                f64::from(i16::from_le_bytes([pcm[at], pcm[at + 1]]))
            })
            .sum::<f64>()
            / channels as f64
    };
    let ratio = f64::from(input_rate) / 24_000.0;
    let cutoff = (1.0 / ratio).min(1.0) * 0.94;
    let mut buffer = Vec::with_capacity(65_536);
    for frame in 0..frames {
        let value = if input_rate == 24_000 {
            sample(frame as usize)
        } else {
            // The same bounded windowed-sinc rule used by script sound copies,
            // extended to long chapters without a material's 120-second limit.
            let at = frame as f64 * ratio;
            let center = at.floor() as i64;
            let mut sum = 0.0;
            let mut weight = 0.0;
            for tap in center - 32..=center + 32 {
                let distance = at - tap as f64;
                if distance.abs() > 32.0 {
                    continue;
                }
                let phase = PI * distance * cutoff;
                let sinc = if phase.abs() < 1e-12 {
                    1.0
                } else {
                    phase.sin() / phase
                };
                let window = 0.42
                    + 0.5 * (PI * distance / 32.0).cos()
                    + 0.08 * (2.0 * PI * distance / 32.0).cos();
                let w = cutoff * sinc * window;
                sum += sample(tap.clamp(0, input_frames as i64 - 1) as usize) * w;
                weight += w;
            }
            sum / weight
        };
        buffer.extend_from_slice(&(value.round().clamp(-32768.0, 32767.0) as i16).to_le_bytes());
        if buffer.len() == 65_536 {
            sink.write(&buffer)?;
            buffer.clear();
        }
    }
    sink.write(&buffer)
}
