use crate::common::*;
use crate::stream::node::{EventReceiver, EventSender, ProcessNode};
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::FFTplanner;
use std::sync::mpsc::channel;

#[derive(Clone)]
struct PsolaInfo {
    in_phase: isize,
    out_phase: isize,
    in_period: isize,
}

#[derive(Getters)]
pub struct PsolaNode {
    receiver: EventReceiver<f32>,
    sender: Option<EventSender<f32>>,
    #[getset(get = "pub", set = "pub")]
    ratio: f32,
    psola_info: Vec<PsolaInfo>,
}

impl PsolaNode {
    pub fn new(receiver: EventReceiver<f32>, ratio: f32) -> Self {
        Self {
            receiver,
            sender: None,
            ratio,
            psola_info: vec![],
        }
    }

    pub fn output(&mut self) -> EventReceiver<f32> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }

    fn psola(&self, data: &[f32], info: &PsolaInfo) -> (Vec<f32>, PsolaInfo) {
        if let Some(in_period) = period(
            data,
            (50, 800),
            data.iter().fold(0.0, |a, b| f32::max(a * a, b * b)) / 4.0,
        ) {
            let ratio = self.ratio;
            let in_phase = info.in_phase;
            let out_phase = info.out_phase;
            let prev_in_period = info.in_period;
            let in_period = in_period as isize;
            let out_period = (in_period as f32 / ratio) as isize;
            let mut in_peak = in_phase + (prev_in_period as isize + in_period) / 2;
            let mut out_peak = out_phase;
            let mut result = vec![0.0; data.len()];
            while out_peak < result.len() as isize {
                while (in_peak + in_period - out_peak).abs() < (in_peak - out_peak).abs()
                    && in_peak + in_period < data.len() as isize
                {
                    in_peak += in_period;
                }
                for d in -in_period..in_period {
                    let i = (out_peak + d) as usize;
                    if i >= result.len() {
                        continue;
                    }
                    let mut in_i = in_peak + d;
                    if in_i < 0 || i >= data.len() {
                        continue;
                    }
                    while in_i >= data.len() as isize {
                        in_i -= in_period
                    }
                    {
                        let in_i = in_i as usize;
                        let in_period = in_period as f32;
                        let d = d as f32;
                        result[i] += data[in_i] * ((in_period - d.abs()) / in_period);
                    }
                }
                out_peak += out_period;
            }

            let info = PsolaInfo {
                in_phase: in_peak - data.len() as isize,
                out_phase: out_peak - result.len() as isize,
                in_period: in_period as isize,
            };
            (result, info)
        } else {
            (data.to_vec(), info.clone())
        }
    }
}

impl ProcessNode<f32> for PsolaNode {
    fn receiver(&self) -> &EventReceiver<f32> {
        &self.receiver
    }

    fn sender(&self) -> Option<EventSender<f32>> {
        self.sender.clone()
    }

    fn process_chunk(&mut self, chunk: SampleChunk<f32>) -> SampleChunk<f32> {
        let channels = *chunk.metadata().channels();
        while self.psola_info.len() < channels {
            self.psola_info.push(PsolaInfo {
                in_phase: 0,
                out_phase: 0,
                in_period: 0,
            });
        }
        let (samples, info) = (0..channels)
            .zip(self.psola_info.iter())
            .map(|(c, info)| self.psola(chunk.samples(c), info))
            .unzip();
        let out_chunk =
            SampleChunk::new(samples, chunk.metadata().clone(), *chunk.duration_samples());
        self.psola_info = info;
        out_chunk
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
