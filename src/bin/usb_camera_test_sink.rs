use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arrow_array::{RecordBatch, StructArray};
use clap::Parser;
use dora_node_api::{DoraArray, DoraNode, Event};
use eyre::{Result, WrapErr, eyre};
use forge_msgs::{CompressedImage, Image};
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(version, about = "Decode and validate every USB camera Dora message")]
struct Cli {
    #[arg(long)]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct SinkConfig {
    #[serde(default = "default_input_id")]
    input_id: String,
    #[serde(default = "default_log_every")]
    log_every: u64,
    #[serde(default = "default_min_frames")]
    min_frames: u64,
    #[serde(default)]
    delay_ms: u64,
}

fn default_input_id() -> String {
    "image".to_string()
}

fn default_log_every() -> u64 {
    30
}

fn default_min_frames() -> u64 {
    1
}

fn require_frames(count: u64, minimum: u64) -> Result<()> {
    if count < minimum {
        eyre::bail!("received {count} valid images; required at least {minimum}");
    }
    Ok(())
}

fn receive_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn should_log(count: u64, log_every: u64) -> bool {
    count == 1 || count.is_multiple_of(log_every.max(1))
}

fn decode_and_describe(data: &DoraArray) -> Result<String> {
    let array = data.as_array();
    let struct_array = array
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| eyre!("input is not an Arrow StructArray"))?;
    let batch = RecordBatch::from(struct_array.clone());

    if let Ok(image) = Image::from_record_batch(&batch) {
        return Ok(format!(
            "message=Image encoding={} width={} height={} bytes={}",
            image.encoding,
            image.width,
            image.height,
            image.data.len()
        ));
    }

    let image = CompressedImage::from_record_batch(&batch)
        .map_err(|err| eyre!("not Image or CompressedImage: {err}"))?;
    let decoded = image
        .to_rgb8_ndarray()
        .map_err(|err| eyre!("failed to decode {} image: {err}", image.format))?;
    let (height, width, _) = decoded.dim();
    Ok(format!(
        "message=CompressedImage encoding={} width={} height={} bytes={}",
        image.format,
        width,
        height,
        image.data.len()
    ))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_text = std::fs::read_to_string(&cli.config)
        .wrap_err_with(|| format!("failed to read {}", cli.config.display()))?;
    let config: SinkConfig = serde_yaml::from_str(&config_text)
        .wrap_err_with(|| format!("failed to parse {}", cli.config.display()))?;
    let (_node, mut events) = DoraNode::init_from_env()?;
    let mut count: u64 = 0;

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, .. } if id.as_str() == config.input_id.as_str() => {
                let description = decode_and_describe(&data)
                    .wrap_err_with(|| format!("invalid image on {}", config.input_id))?;
                count += 1;
                if should_log(count, config.log_every) {
                    let received_at_ms = receive_time_ms();
                    println!("count={count} received_at_unix_ms={received_at_ms} {description}");
                }
                if config.delay_ms > 0 {
                    std::thread::sleep(Duration::from_millis(config.delay_ms));
                }
            }
            Event::Stop(_) => break,
            Event::Error(error) => return Err(eyre!("Dora error: {error}")),
            _ => {}
        }
    }
    require_frames(count, config.min_frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use dora_node_api::IntoArrow;

    #[test]
    fn no_frames_is_not_a_successful_stream_test() {
        assert!(require_frames(0, 1).is_err());
        assert!(require_frames(1, 1).is_ok());
    }

    #[test]
    fn raw_and_depth_images_are_validated() {
        for (encoding, step, bytes) in [
            ("rgb8", 3, vec![1, 2, 3]),
            ("32FC1", 4, 1.25_f32.to_le_bytes().to_vec()),
        ] {
            let image = Image::new(1, 1, encoding, step, Bytes::from(bytes)).unwrap();
            let array: StructArray = image.to_record_batch().unwrap().into();
            assert!(decode_and_describe(&array.into_arrow()).is_ok());
        }
    }

    #[test]
    fn malformed_input_is_rejected() {
        let array = arrow_array::UInt8Array::from(vec![1_u8, 2, 3]);
        assert!(decode_and_describe(&array.into_arrow()).is_err());
    }
}
