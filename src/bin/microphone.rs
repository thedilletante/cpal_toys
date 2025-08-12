use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use anyhow::Context;


fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device().context("Failed to get default input device")?;
    let config = device.default_input_config().context("Failed to get default input config")?;
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Process the audio data here
            println!("Received {} samples: {:?}", data.len(), data[..20].to_vec());
        },
        move |err| {
            eprintln!("An error occurred on the input stream: {}", err);
        },
        None,
    )?;
    stream.play().context("Failed to start the input stream")?;

    std::thread::sleep(std::time::Duration::from_secs(5)); // Keep the stream alive for 5 seconds

    Ok(())
}
