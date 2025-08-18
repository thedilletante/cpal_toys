use std::time::Duration;
use std::collections::VecDeque;

pub struct Window {
    buffer: VecDeque<f32>,
    size: usize,
}

impl Window {
    pub fn with_duration(duration: Duration, sample_rate: u32) -> Self {
        let size = (duration.as_secs_f64() * sample_rate as f64) as usize;
        Self {
            buffer: VecDeque::with_capacity(size),
            size,
        }
    }

    pub fn add_samples(&mut self, samples: &[f32]) {
        if samples.len() > self.size {
            self.buffer.clear();
            // If samples exceed the size, truncate to the last `size` samples
            let to_add = &samples[samples.len() - self.size..];
            self.buffer.extend(&samples[samples.len() - self.size..]);
            return;
        }

        while self.buffer.len() + samples.len() > self.size {
            self.buffer.pop_front();
        }

        self.buffer.extend(samples);
    }

    pub fn is_ready(&self) -> bool {
        self.buffer.len() == self.size
    }

    pub fn calculate_rms(&self) -> Option<f32> {
        if !self.is_ready() {
            return None;
        }
        let sum_of_squares: f32 = self.buffer.iter().map(|&x| x * x).sum();
        let rms = (sum_of_squares / self.buffer.len() as f32).sqrt();
        Some(rms)
    }

    pub fn calculate_dbfs(&self) -> Option<f32> {
        // add epsilon to avoid log(0)
        let rms = self.calculate_rms()? + 1e-10;
        Some(20.0 * rms.log10())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_window_creation() {
        let mut window = Window::with_duration(Duration::from_millis(20), 44100);
        assert_eq!(window.size, 882);
        assert!(window.buffer.is_empty());
        assert!(!window.is_ready());

        window.add_samples(&[0.0; 882]);
        assert!(window.is_ready());
        assert_eq!(window.calculate_rms(), Some(0.0));
        assert_eq!(window.calculate_dbfs(), Some(-200.0));
    }

}
