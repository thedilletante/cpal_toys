use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashMap;
use anyhow::Context;
use lockfree::queue::Queue;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HashableStreamConfig {
    channels: u16,
    sample_rate: u32,
    sample_format: cpal::SampleFormat,
}

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let devices = host.devices()?;

    let mut input_devices = Vec::new();
    let mut output_devices = Vec::new();

    for device in devices {
        if device.supports_input() {
            input_devices.push(device.clone());
        }
        if device.supports_output() {
            output_devices.push(device.clone());
        }
    }

    let mut config_to_devices: HashMap<HashableStreamConfig, (cpal::SupportedStreamConfig, Vec<cpal::Device>)> = Default::default();

    for device in input_devices {
        let configs = all_input_configs(&device)?;
        for config in configs {
            let hashable_config = HashableStreamConfig {
                channels: config.channels(),
                sample_rate: config.sample_rate().0,
                sample_format: config.sample_format(),
            };
            config_to_devices.entry(hashable_config)
                .and_modify(|e| e.1.push(device.clone()))
                .or_insert((config, vec![device.clone()]));
        }
    }

    let mut config_with_two_devices = config_to_devices
        .into_iter()
        .find(|(_, (_, devices))| devices.len() >= 2)
        .map(|(_, v)| v)
        .context("No input devices with identical configurations found")?;

    let config = config_with_two_devices
        .0;

    let left_device = config_with_two_devices.1.pop().context("Expected at least two devices")?;
    let right_device = config_with_two_devices.1.pop().context("Expected at least two devices")?;

    println!("Left device is {} and right device is {}", left_device.name()?, right_device.name()?);

    let mut output: Option<(cpal::Device, cpal::SupportedStreamConfig)> = None;

    for device in output_devices {
        let configs = all_output_configs(&device)?;
        for config_out in configs {
            if config_out.sample_rate() == config.sample_rate() && config_out.channels() == 2 {
                output = Some((device, config_out));
                break;
            }
        }
    }

    let (output_device, output_config) = output
        .context("No suitable output device found with 2 channels and matching sample rate")?;

    println!("Output device is {}", output_device.name()?);

    let mut input_config: cpal::StreamConfig = config.into();
    let mut output_config: cpal::StreamConfig = output_config.into();
    input_config.buffer_size = cpal::BufferSize::Fixed(15);
    output_config.buffer_size = cpal::BufferSize::Fixed(15);

    let left_queue = Arc::new(Queue::new());
    let left_queue_input = left_queue.clone();
    let left_stream = left_device.build_input_stream(
        &input_config,
        move |data: &[f32], _| {
            while let Some(_) = left_queue_input.pop() {
                // Clear the queue if it has any data
            }
            left_queue_input.push(data.to_vec());
        },
        move |err| {
            eprintln!("Error on left input stream: {}", err);
        },
        None,
    )?;

    let right_queue = Arc::new(Queue::new());
    let right_queue_input = right_queue.clone();
    let right_stream = right_device.build_input_stream(
        &input_config,
        move |data: &[f32], _| {
            while let Some(_) = right_queue_input.pop() {
                // Clear the queue if it has any data
            }
            right_queue_input.push(data.to_vec());
        },
        move |err| {
            eprintln!("Error on right input stream: {}", err);
        },
        None,
    )?;

    let output_stream = output_device.build_output_stream(
        &output_config,
        move |data: &mut [f32], _| {
            let left_chunk = left_queue.pop();
            let right_chunk = right_queue.pop();

            for ((left, right), out) in left_chunk.iter().flatten().zip(right_chunk.iter().flatten()).zip(data.chunks_exact_mut(2)) {
                out[0] = *left;
                out[1] = *right;
            }

        },
        move |err| {
            eprintln!("Error on output stream: {}", err);
        },
        None,
    )?;

    left_stream.play()?;
    right_stream.play()?;
    output_stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs(60)); // Keep the streams alive for 60
                                                            // seconds


    Ok(())
}

fn all_input_configs(device: &cpal::Device) -> anyhow::Result<Vec<cpal::SupportedStreamConfig>> {
    let default_config = device.default_input_config()?;
    let supported_configs = device.supported_input_configs()?;
    let mut configs = Vec::new();
    configs.push(default_config.clone());
    for config in supported_configs {
        let supported_config = config.try_with_sample_rate(config.max_sample_rate()).context("Failed to make stream config")?;
        if supported_config == default_config {
            continue; // Skip the default config, as it's already included
        }
        configs.push(supported_config);
    }
    Ok(configs)
}

fn all_output_configs(device: &cpal::Device) -> anyhow::Result<Vec<cpal::SupportedStreamConfig>> {
    let default_config = device.default_output_config()?;
    let supported_configs = device.supported_output_configs()?;
    let mut configs = Vec::new();
    configs.push(default_config.clone());
    for config in supported_configs {
        let supported_config = config.try_with_sample_rate(config.max_sample_rate()).context("Failed to make stream config")?;
        if supported_config == default_config {
            continue; // Skip the default config, as it's already included
        }
        configs.push(supported_config);
    }
    Ok(configs)
}
