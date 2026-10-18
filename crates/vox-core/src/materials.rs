//! Local procedural sounds and an explicit, bounded script-format conversion.
use crate::{
    AudioOriginDisclosure,
    audio::{SpeechWaveAssembly, decode_pcm_s16le_wav},
    project::{Error, Result},
};
use std::f64::consts::PI;
pub const BUILTIN_SOUNDS: &[&str] = &[
    "soft_chime",
    "double_beep",
    "click",
    "wood_block",
    "bell",
    "success",
    "whoosh",
    "countdown",
];
fn wav(pcm: &[u8]) -> Result<Vec<u8>> {
    let mut wave = SpeechWaveAssembly::default();
    wave.push_pcm(24_000, 1, pcm)?;
    Ok(wave.finish()?)
}
pub fn builtin_wav(key: &str) -> Result<Vec<u8>> {
    let duration = match key {
        "click" => 0.08,
        "wood_block" => 0.18,
        "bell" => 1.2,
        "countdown" => 1.8,
        "whoosh" => 0.7,
        "soft_chime" | "double_beep" | "success" => 0.8,
        _ => return Err(Error::new("invalid_material")),
    };
    let frames = (duration * 24_000.0) as usize;
    let mut pcm = Vec::with_capacity(frames * 2);
    let mut noise = 0x37ab21u32;
    for i in 0..frames {
        let t = i as f64 / 24_000.0;
        noise = noise.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let n = f64::from(noise >> 8) / 8_388_608.0 - 1.0;
        let tone = |frequency: f64, time: f64| (2.0 * PI * frequency * time).sin();
        let (v, env) = match key {
            "click" => (n, (-t * 90.0).exp()),
            "wood_block" => (tone(760.0, t) + 0.35 * tone(1390.0, t), (-t * 30.0).exp()),
            "bell" => (
                tone(880.0, t) + 0.35 * tone(1763.0, t) + 0.15 * tone(2635.0, t),
                (-t * 4.5).exp(),
            ),
            "whoosh" => (n, (PI * t / duration).sin().powi(2) * 0.5),
            "double_beep" => {
                let x = if t < 0.4 { t } else { t - 0.4 };
                (
                    tone(740.0, x),
                    if x < 0.16 {
                        (PI * x / 0.16).sin().powi(2)
                    } else {
                        0.0
                    },
                )
            }
            "countdown" => {
                let x = t % 0.6;
                (
                    tone(if t < 1.2 { 660.0 } else { 990.0 }, x),
                    if x < 0.15 {
                        (PI * x / 0.15).sin().powi(2)
                    } else {
                        0.0
                    },
                )
            }
            "success" => {
                let x = if t < 0.25 { t } else { t - 0.25 };
                (
                    tone(if t < 0.25 { 660.0 } else { 880.0 }, x),
                    (-x * 8.0).exp(),
                )
            }
            _ => (tone(659.25, t) + 0.45 * tone(987.77, t), (-t * 6.0).exp()),
        };
        let edge = (t / 0.008).min(1.0) * ((duration - t) / 0.012).min(1.0);
        let sample = (v * env * edge * 6500.0).clamp(-32768.0, 32767.0).round() as i16;
        pcm.extend_from_slice(&sample.to_le_bytes());
    }
    wav(&pcm)
}
/// Retains the original separately; downmix then apply a windowed sinc low-pass
/// before 24 kHz resampling. No implicit format changes in the script compiler.
pub fn script_wav(bytes: &[u8]) -> Result<Vec<u8>> {
    let (contract, pcm) = decode_pcm_s16le_wav(bytes, AudioOriginDisclosure::ImportedUnverified)?;
    if contract.channels > 2 || contract.duration_millis() > 120_000 {
        return Err(Error::new("sound_too_long"));
    }
    if contract.channels == 1 && contract.sample_rate_hz == 24_000 {
        return Ok(bytes.to_vec());
    }
    let channels = usize::from(contract.channels);
    let samples: Vec<f64> = pcm
        .chunks_exact(channels * 2)
        .map(|f| {
            f.chunks_exact(2)
                .map(|s| f64::from(i16::from_le_bytes([s[0], s[1]])))
                .sum::<f64>()
                / channels as f64
        })
        .collect();
    let frames = contract.frame_count * 24_000 / u64::from(contract.sample_rate_hz);
    let mut output = Vec::with_capacity(frames as usize * 2);
    let ratio = f64::from(contract.sample_rate_hz) / 24_000.0;
    let cutoff = (1.0 / ratio).min(1.0) * 0.94;
    for i in 0..frames {
        let at = i as f64 * ratio;
        let center = at.floor() as i64;
        let mut sum = 0.0;
        let mut weight = 0.0;
        for j in center - 32..=center + 32 {
            let d = at - j as f64;
            if d.abs() > 32.0 {
                continue;
            }
            let x = PI * d * cutoff;
            let sinc = if x.abs() < 1e-12 { 1.0 } else { x.sin() / x };
            let window = 0.42 + 0.5 * (PI * d / 32.0).cos() + 0.08 * (2.0 * PI * d / 32.0).cos();
            let w = cutoff * sinc * window;
            let sample = samples[j.clamp(0, samples.len() as i64 - 1) as usize];
            sum += sample * w;
            weight += w;
        }
        let value = (sum / weight).round().clamp(-32768.0, 32767.0) as i16;
        output.extend_from_slice(&value.to_le_bytes());
    }
    wav(&output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin_catalog_is_bounded_and_repeatable() {
        for key in BUILTIN_SOUNDS {
            let a = builtin_wav(key).unwrap();
            assert_eq!(a, builtin_wav(key).unwrap());
            crate::assembly::validate_cue(&a).unwrap();
        }
    }
    #[test]
    fn conversion_preserves_duration_and_downmixes_opposite_channels() {
        let mut wave = SpeechWaveAssembly::default();
        let mut pcm = vec![];
        for _ in 0..4410 {
            pcm.extend_from_slice(&10000i16.to_le_bytes());
            pcm.extend_from_slice(&(-10000i16).to_le_bytes());
        }
        wave.push_pcm(44_100, 2, &pcm).unwrap();
        let converted = script_wav(&wave.finish().unwrap()).unwrap();
        let (c, p) =
            decode_pcm_s16le_wav(&converted, AudioOriginDisclosure::ImportedUnverified).unwrap();
        assert_eq!(
            (c.sample_rate_hz, c.channels, c.frame_count),
            (24000, 1, 2400)
        );
        assert!(p.iter().all(|b| *b == 0));
    }
}
