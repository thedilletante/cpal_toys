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
            self.buffer.extend(&samples[samples.len() - self.size..]);
            return;
        }

        if self.buffer.len() + samples.len() > self.size {
            // If adding these samples would exceed the size, we need to remove some from the front
            let excess = self.buffer.len() + samples.len() - self.size;
            if excess > 0 {
                self.buffer.drain(0..excess.min(self.buffer.len()));
            }
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

    // Returns the frequency spectrum of the window
    // using FFT
    // from 20Hz to 20kHz
    // Every element is a pair of the frequency and its center of mass
    pub fn calculate_frequencies(&self) -> Option<Vec<(f64, f64)>> {
        if !self.is_ready() {
            return None;
        }

        let n = self.buffer.len();

        let mut frequencies = Vec::with_capacity(n / 2);
        for k in 0..n / 2 {
            let mut real = 0.0;
            let mut imag = 0.0;

            for (i, &sample) in self.buffer.iter().enumerate() {
                let angle = 2.0 * std::f64::consts::PI * (k as f64 * i as f64 / n as f64);
                real += sample as f64 * angle.cos();
                imag -= sample as f64 * angle.sin(); // Note: imaginary part is negated
            }

            let magnitude = (real * real + imag * imag).sqrt();
            frequencies.push((k as f64, magnitude));
        }
        Some(frequencies)
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

        window.add_samples(&[1.0; 441]);
        assert!(window.is_ready());
        assert_eq!(window.calculate_rms(), Some(0.70710677)); // sqrt(1/2)
        assert_eq!(window.calculate_dbfs(), Some(-3.0103002));
    }

}
