use derive_new::new;
use getset::{Getters, Setters};
use rustfft::num_traits::{FromPrimitive, Num, NumAssignOps, NumCast, NumOps};
use rustfft::num_complex::Complex32;
use serde::{Serialize, Deserialize};

#[derive(Getters, Clone, Debug, new)]
#[getset(get = "pub")]
pub struct AudioMetadata {
    channels: usize,
    sample_rate: usize,
}

#[derive(Debug, Clone)]
struct SampleLengthError;

impl std::fmt::Display for SampleLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid number of samples")
    }
}

impl std::error::Error for SampleLengthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait Sample:
    Num + NumAssignOps + NumOps + NumCast + FromPrimitive + Clone + Send + Sync + Copy
{
}

impl Sample for f32 {}
impl Sample for Complex32 {}

#[derive(Getters, Setters, Clone, Debug, new)]
pub struct SampleChunk<S: Sample> {
    samples: Vec<Vec<S>>,
    #[getset(get = "pub")]
    metadata: AudioMetadata,
    #[getset(get = "pub")]
    duration_samples: usize,
    #[getset(get = "pub", set = "pub")]
    window_info: Option<WindowInfo>,
}

impl<S: Sample> SampleChunk<S> {
    pub fn from_flat_samples(
        flat_samples: &[S],
        metadata: AudioMetadata,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let len = flat_samples.len();
        let channels = *metadata.channels();
        let duration_samples = len / channels;
        if duration_samples * channels != len {
            return Err(Box::new(SampleLengthError));
        }
        let mut samples = vec![vec![]; channels];
        for i in 0..duration_samples {
            for channel in 0..channels {
                samples[channel].push(flat_samples[i * channels + channel].clone());
            }
        }
        Ok(Self {
            samples,
            metadata,
            duration_samples,
            window_info: None,
        })
    }

    pub fn truncate(&mut self, duration: usize) {
        for channel in self.samples.iter_mut() {
            channel.truncate(duration);
        }
        self.duration_samples = duration;
    }

    pub fn samples(&self, channel: usize) -> &[S] {
        &self.samples[channel]
    }

    pub fn samples_mut(&mut self, channel: usize) -> &mut [S] {
        &mut self.samples[channel]
    }

    pub fn flattened_samples(&self) -> Vec<S> {
        let channels = *self.metadata().channels();
        if channels == 1 {
            return self.samples[0].clone();
        }
        let mut flattened = vec![];
        for i in 0..self.duration_samples {
            for channel in 0..channels {
                flattened.push(self.samples(channel)[i].clone());
            }
        }
        flattened
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WindowFunction {
    Hanning,
    Triangular,
    Rectangular
}

#[derive(Getters, Clone, Debug, new)]
pub struct WindowInfo {
    #[getset(get = "pub")]
    window_function: WindowFunction,
    #[getset(get = "pub")]
    delay: usize,
}

