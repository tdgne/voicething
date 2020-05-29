use derive_new::new;
use getset::Getters;
use rustfft::num_traits::{FromPrimitive, Num, NumAssignOps, NumCast, NumOps};
use rustfft::num_complex::Complex32;

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

#[derive(Getters, Clone, Debug, new)]
pub struct SampleChunk<S: Sample> {
    samples: Vec<Vec<S>>,
    #[getset(get = "pub")]
    metadata: AudioMetadata,
    #[getset(get = "pub")]
    duration_samples: usize,
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

#[derive(Clone, Debug)]
pub enum WindowFunction {
    Hanning,
    Rectangular
}

#[derive(Getters, Clone, Debug, new)]
pub struct WindowedSampleChunk<S: Sample> {
    #[getset(get = "pub")]
    chunk: SampleChunk<S>,
    #[getset(get_copy = "pub")]
    window_function: WindowFunction,
    #[getset(get_copy = "pub")]
    delay: usize,
}

pub trait Chunk<S: Sample>: Clone {}
impl<S: Sample> Chunk<S> for SampleChunk<S> {}
impl<S: Sample> Chunk<S> for WindowedSampleChunk<S> {}

