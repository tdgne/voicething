use derive_new::new;
use getset::{Getters, Setters};
use rustfft::num_complex::Complex32;
use rustfft::num_traits::{FromPrimitive, Num, NumAssignOps, NumCast, NumOps};
use serde::{Deserialize, Serialize};

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
pub struct GenericDataChunk<S: Sample> {
    samples: Vec<Vec<S>>,
    #[getset(get = "pub")]
    metadata: AudioMetadata,
    #[getset(get = "pub")]
    duration: usize,
    #[getset(get = "pub", set = "pub")]
    window_info: Option<WindowInfo>,
}

impl<S: Sample> GenericDataChunk<S> {
    pub fn from_flat_sata(
        flat_sata: &[S],
        metadata: AudioMetadata,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let len = flat_sata.len();
        let channels = *metadata.channels();
        let duration = len / channels;
        if duration * channels != len {
            return Err(Box::new(SampleLengthError));
        }
        let mut samples = vec![vec![]; channels];
        for i in 0..duration {
            for channel in 0..channels {
                samples[channel].push(flat_sata[i * channels + channel].clone());
            }
        }
        Ok(Self {
            samples,
            metadata,
            duration,
            window_info: None,
        })
    }

    pub fn truncate(&mut self, duration: usize) {
        for channel in self.samples.iter_mut() {
            channel.truncate(duration);
        }
        self.duration = duration;
    }

    pub fn samples(&self, channel: usize) -> &[S] {
        &self.samples[channel]
    }

    pub fn samples_mut(&mut self, channel: usize) -> &mut [S] {
        &mut self.samples[channel]
    }

    pub fn flattened_data(&self) -> Vec<S> {
        let channels = *self.metadata().channels();
        if channels == 1 {
            return self.samples[0].clone();
        }
        let mut flattened = vec![];
        for i in 0..self.duration {
            for channel in 0..channels {
                flattened.push(self.samples(channel)[i].clone());
            }
        }
        flattened
    }
}

#[derive(Clone, Debug)]
pub enum DataChunk {
    Real(GenericDataChunk<f32>),
    Complex(GenericDataChunk<Complex32>),
}

impl DataChunk {
    pub fn metadata(&self) -> &AudioMetadata {
        match self {
            Self::Real(c) => c.metadata(),
            Self::Complex(c) => c.metadata(),
        }
    }

    pub fn duration(&self) -> &usize {
        match self {
            Self::Real(c) => c.duration(),
            Self::Complex(c) => c.duration(),
        }
    }

    pub fn window_info(&self) -> &Option<WindowInfo> {
        match self {
            Self::Real(c) => c.window_info(),
            Self::Complex(c) => c.window_info(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WindowFunction {
    Hanning,
    Triangular,
    Rectangular,
}

#[derive(Getters, Clone, Debug, new)]
pub struct WindowInfo {
    #[getset(get = "pub")]
    window_function: WindowFunction,
    #[getset(get = "pub")]
    delay: usize,
}
