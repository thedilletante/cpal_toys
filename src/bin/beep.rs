use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample, I24,
};

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
        cpal::SampleFormat::I24 => run::<I24>(&device, &mut config.into()),
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
    T: SizedSample + FromSample<f32>,
{
    config.buffer_size = cpal::BufferSize::Fixed(4096);
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value_left = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        let freq = if sample_clock > sample_rate / 2.0 { 440.0 } else { 490.0 };
        (sample_clock * freq * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };
    let mut next_value_right = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        let freq = if sample_clock > sample_rate / 2.0 { 550.0 } else { 510.0 };
        (sample_clock * freq * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
            println!("Callback info {:?}", info);
            write_data(data, channels, &mut next_value_left, &mut next_value_right)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(8000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample_left: &mut dyn FnMut() -> f32, next_sample_right: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        assert_eq!(frame.len(), 2);
        let value_left: T = T::from_sample(next_sample_left());
        let value_right: T = T::from_sample(next_sample_right());
        frame[0] = value_left;
        frame[1] = value_right;
    }
}
