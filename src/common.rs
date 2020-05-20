use derive_new::new;
use getset::Getters;
use rustfft::num_traits::Num;

#[derive(Getters)]
#[getset(get = "pub")]
#[derive(new)]
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

#[derive(Getters)]
pub struct SampleChunk<S: Num + Clone> {
    samples: Vec<Vec<S>>,
    #[getset(get = "pub")]
    metadata: AudioMetadata,
    duration_samples: usize,
}

impl<S: Num + Clone> SampleChunk<S> {
    pub fn from_flat_samples(
        flat_samples: &[S],
        metadata: AudioMetadata,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let len = flat_samples.len();
        let duration_samples = len / metadata.channels();
        if duration_samples * metadata.channels() != len {
            return Err(Box::new(SampleLengthError));
        }
        let samples = flat_samples
            .chunks_exact(duration_samples)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>();
        Ok(Self {
            samples,
            metadata,
            duration_samples,
        })
    }

    pub fn samples(&self, channel: usize) -> &[S] {
        &self.samples[channel]
    }
}
