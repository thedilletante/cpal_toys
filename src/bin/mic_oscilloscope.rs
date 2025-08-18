use ratatui::prelude::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Dataset, GraphType, Axis, Chart, Gauge, Borders},
    symbols::Marker,
    Frame,
};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use anyhow::Context;
use lockfree::queue::Queue;
use std::sync::Arc;
use cpal_toy::window::Window;

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device().context("Failed to get default input device")?;
    let config = device.default_input_config().context("Failed to get default input config")?;
    let sample_rate = config.sample_rate().0;
    let left_queue = Arc::new(Queue::new());
    let left_queue_input = left_queue.clone();
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            left_queue_input.push(data.to_vec());
        },
        move |err| {
            eprintln!("An error occurred on the input stream: {}", err);
        },
        None,
    )?;
    stream.play().context("Failed to start the input stream")?;
    let mut terminal = ratatui::init();
    let window = Window::with_duration(std::time::Duration::from_millis(100), sample_rate);
    let result = run(&mut terminal, left_queue, sample_rate as usize * 2, window);
    ratatui::restore();
    result.context("Failed to run the oscilloscope")
}

fn run(terminal: &mut ratatui::DefaultTerminal, queue: Arc<Queue<Vec<f32>>>, total_samples: usize, mut window: Window) -> std::io::Result<()> {
    let mut samples: Vec<f32> = vec![0.0; total_samples];
    let mut last_timeout = std::time::Instant::now();
    loop {
        while let Some(data) = queue.pop() {
            window.add_samples(&data);
            samples.extend(data);
            if samples.len() > total_samples {
                samples.drain(0..samples.len() - total_samples); // Keep the last 1000 samples
            }
        }
        terminal.draw(|f| draw(f, &samples, total_samples, &window))?;
        let next_tick = last_timeout + std::time::Duration::from_secs_f32(1.0 / 30.0);
        if let Ok(true) = event::poll(next_tick.duration_since(std::time::Instant::now())) {
            if handle_events()? {
                break;
            }
        } else {
            last_timeout = std::time::Instant::now();
        }
    }
    Ok(())
}

fn draw(frame: &mut ratatui::Frame, samples: &Vec<f32>, total_samples: usize, window: &Window) {
    use Constraint::{Fill, Length, Min};

    let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(3), Constraint::Fill(1)]).spacing(1);
    let [top, dbfs_area, oscilloscope_area] = layout.areas(frame.area());

    let title = Line::from_iter([
        Span::from("Oscilloscope").bold(),
        Span::from(" (Press 'q' to quit)"),
    ]);
    frame.render_widget(title.centered(), top);

    let data = samples.iter().enumerate().map(|(i, &sample)| {
        let x = (i as f64 / total_samples as f64) * 2000.0; // Scale x to 2000ms
        let y = sample as f64; // Use sample value directly for y
        (x, y)
    }).collect::<Vec<_>>();

    let dataset = Dataset::default()
        .name("Amplitude")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Color::Green)
        .data(data.as_ref());

    let x_axis = Axis::default()
        .title("Time".blue())
        .bounds([0.0, 2000.0])
        .labels(["0", "1s", "2s"]);

    let y_axis = Axis::default()
        .title("Amplitude".blue())
        .bounds([-1.0, 1.0])
        .labels(["-1", "0", "1"]);

    let chart = Chart::new(vec![dataset]).x_axis(x_axis).y_axis(y_axis);
    frame.render_widget(chart, oscilloscope_area);

    let dbfs_percent = {
        if let Some(dbfs) = window.calculate_dbfs() {
            let low_limit = -40.0; // dBFS low limit
            let value = dbfs.min(0.0).max(low_limit);
            (value - low_limit) / (-low_limit) // Normalize to 0.0 - 1.0
        } else {
            0.0
        }
    };
    let dbfs_gauge = Gauge::default()
        .block(Block::default().title("dBFS").borders(Borders::ALL))
        .gauge_style(Color::Cyan)
        .ratio(dbfs_percent as f64);
    frame.render_widget(dbfs_gauge, dbfs_area);
}

fn handle_events() -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return Ok(true),
            // handle other key events
            _ => {}
        },
        // handle other events
        _ => {}
    }
    Ok(false)
}
