use crate::common::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::FFTplanner;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone)]
struct PsolaInfo {
    old_phase: isize,
    new_phase: isize,
    prev_old_period: isize,
}

#[derive(Getters)]
pub struct PsolaNode {
    #[getset(get = "pub", set = "pub")]
    receiver: Receiver<SampleChunk<f32>>,
    sender: Option<Sender<SampleChunk<f32>>>,
    #[getset(get = "pub", set = "pub")]
    ratio: f32,
    psola_info: Vec<PsolaInfo>,
}

impl PsolaNode {
    pub fn new(receiver: Receiver<SampleChunk<f32>>, ratio: f32) -> Self {
        Self {
            receiver,
            sender: None,
            ratio,
            psola_info: vec![],
        }
    }

    pub fn run(&mut self) {
        for chunk in self.receiver.iter() {
            if let Some(ref sender) = self.sender {
                let channels = *chunk.metadata().channels();
                while self.psola_info.len() < channels {
                    self.psola_info.push(PsolaInfo {
                        old_phase: 0,
                        new_phase: 0,
                        prev_old_period: 0,
                    });
                }
                let (samples, info) = (0..channels)
                    .zip(self.psola_info.iter())
                    .map(|(c, info)| self.psola(chunk.samples(c), info))
                    .unzip();
                let new_chunk = SampleChunk::new(
                    samples,
                    chunk.metadata().clone(),
                    *chunk.duration_samples(),
                );
                self.psola_info = info;
                sender.send(new_chunk).expect("channel broken");
            }
        }
    }

    pub fn output(&mut self) -> Receiver<SampleChunk<f32>> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }

    fn psola(&self, data: &[f32], info: &PsolaInfo) -> (Vec<f32>, PsolaInfo) {
        if let Some(old_period) = period(
            data,
            (50, 800),
            data.iter().fold(0.0, |a, b| f32::max(a * a, b * b)) / 4.0,
        ) {
            let ratio = self.ratio;
            let old_phase = info.old_phase;
            let new_phase = info.new_phase;
            let prev_old_period = info.prev_old_period;
            let old_period = old_period as isize;
            let new_period = (old_period as f32 / ratio) as isize;
            let mut old_peak = old_phase + (prev_old_period as isize + old_period) / 2;
            let mut new_peak = new_phase;
            let mut result = vec![0.0; data.len()];
            while new_peak < result.len() as isize {
                while (old_peak + old_period - new_peak).abs() < (old_peak - new_peak).abs()
                    && old_peak + old_period < data.len() as isize
                {
                    old_peak += old_period;
                }
                for d in -old_period..old_period {
                    let i = (new_peak + d) as usize;
                    if i >= result.len() {
                        continue;
                    }
                    let mut old_i = old_peak + d;
                    if old_i < 0 || i >= data.len() {
                        continue;
                    }
                    while old_i >= data.len() as isize {
                        old_i -= old_period
                    }
                    {
                        let old_i = old_i as usize;
                        let old_period = old_period as f32;
                        let d = d as f32;
                        result[i] += data[old_i] * ((old_period - d.abs()) / old_period);
                    }
                }
                new_peak += new_period;
            }

            let info = PsolaInfo {
                old_phase: old_peak - data.len() as isize,
                new_phase: new_peak - result.len() as isize,
                prev_old_period: old_period as isize,
            };
            (result, info)
        } else {
            (data.to_vec(), info.clone())
        }
    }
}

fn period(
    data: &[f32],
    period_lims: (usize, usize),
    minimum_autocorrelation: f32,
) -> Option<usize> {
    // calculate autocorrelation using FFT
    let fft = {
        let mut planner = FFTplanner::new(false);
        planner.plan_fft(data.len())
    };
    let ifft = {
        let mut planner = FFTplanner::new(true);
        planner.plan_fft(data.len())
    };
    let mut signal = data
        .iter()
        .map(|d| Complex32::new(*d, 0.0))
        .collect::<Vec<_>>();
    let mut spectrum = signal.clone();
    fft.process(&mut signal, &mut spectrum);
    spectrum[0] = Complex32::new(0.0, 0.0); // ignore DC content
    let mut prod = spectrum.iter().map(|c| *c * c.conj()).collect::<Vec<_>>();
    let mut coefs = signal.clone();
    ifft.process(&mut prod, &mut coefs);

    let mut max = 0.0;
    let mut period = None;
    for (i, c) in coefs.iter().map(|c| c.re).enumerate() {
        if max < c && minimum_autocorrelation < c && period_lims.0 <= i && i <= period_lims.1 {
            max = c;
            period = Some(i);
        }
    }
    period
}
