use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample, I24,
};

use cpal_toy::{TonePlayerConfigBuilder, TonePlayer};

fn main() -> anyhow::Result<()> {

    let host = cpal::default_host();

    let device = host.default_output_device().ok_or(anyhow::anyhow!("Failed to get default device"))?;
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {config:?}");
    println!("Calling from {:?}", std::thread::current().id());

    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &mut config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &mut config.into()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &mut config.into()),
        // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &mut config.into()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &mut config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &mut config.into()),
        // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &mut config.into()),
        // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &mut config.into()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &mut config.into()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &mut config.into()),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

pub fn run<T>(device: &cpal::Device, config: &mut cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32> + std::ops::AddAssign
{
    config.buffer_size = cpal::BufferSize::Fixed(32);
    let mut player1 = TonePlayer::with_config(
        TonePlayerConfigBuilder::default()
            .frequency(440.0)
            .sample_rate(config.sample_rate.0 as u32)
            .channels(config.channels as usize)
            .build()?,
    );
    let mut player2 = TonePlayer::with_config(
        TonePlayerConfigBuilder::default()
            .frequency(880.0)
            .factor(0.5)
            .sample_rate(config.sample_rate.0 as u32)
            .channels(config.channels as usize)
            .mix(true)
            .build()?,
    );
    let mut player3 = TonePlayer::with_config(
        TonePlayerConfigBuilder::default()
            .frequency(1320.0)
            .factor(0.5)
            .sample_rate(config.sample_rate.0 as u32)
            .channels(config.channels as usize)
            .mix(true)
            .build()?,
    );

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, &mut player1, &mut player2, &mut player3);
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(8000));

    Ok(())
}

fn write_data<T>(output: &mut [T], player1: &mut TonePlayer, player2: &mut TonePlayer, player3: &mut TonePlayer)
where
    T: Sample + FromSample<f32> + std::ops::AddAssign
{
    player1.fill_buffer(output);
    player2.fill_buffer(output);
    player3.fill_buffer(output);
}
