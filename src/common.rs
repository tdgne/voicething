use derive_new::new;
use getset::Getters;
use rustfft::num_traits::Num;

#[derive(Getters, Clone, new)]
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

#[derive(Getters, Clone, new)]
pub struct SampleChunk<S: Num + Clone> {
    samples: Vec<Vec<S>>,
    #[getset(get = "pub")]
    metadata: AudioMetadata,
    #[getset(get = "pub")]
    duration_samples: usize,
}

impl<S: Num + Clone> SampleChunk<S> {
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

    pub fn samples(&self, channel: usize) -> &[S] {
        &self.samples[channel]
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
