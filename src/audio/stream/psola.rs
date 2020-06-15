use super::super::common::*;
use super::node::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::FFTplanner;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
struct PsolaInfo {
    last_in_center: isize,
    last_out_center: isize,
    in_period: isize,
}

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct PsolaNode {
    io: NodeIo,
    ratio: f32,
    #[serde(skip)]
    psola_info: Vec<PsolaInfo>,
    id: NodeId,
}

impl HasNodeIo for PsolaNode {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl NodeTrait for PsolaNode {
    fn id(&self) -> NodeId {
        self.id
    }
    fn run_once(&mut self) {
        if self.inputs().len() != 1 {
            return;
        }
        while let Some(chunk) = self.inputs()[0].try_recv().ok() {
            if let Some(chunk) = self.process_chunk(chunk) {
                for output in self.outputs().iter() {
                    let _ = output.try_send(chunk.clone());
                }
            }
        }
    }
}

impl PsolaNode {
    pub fn new(ratio: f32) -> Self {
        Self {
            io: NodeIo::new(),
            ratio,
            psola_info: vec![],
            id: NodeId::new(),
        }
    }

    pub fn ratio(&mut self) -> f32 {
        self.ratio
    }

    pub fn ratio_mut(&mut self) -> &mut f32 {
        &mut self.ratio
    }

    pub fn period(
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

    fn triangular_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        1.0 - (x - 0.5).abs() * 2.0
    }

    fn hanning_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        0.5 - 0.5 * (2.0 * 3.141592 * x).cos()
    }

    fn unitary_ola(
        data: &[f32],
        result: &mut [f32],
        in_center: isize,
        in_period: usize,
        out_center: isize,
    ) {
        for i in in_center - in_period as isize..in_center + in_period as isize {
            let amplitude = {
                let mut i = i;
                while i < 0 {
                    i += in_period as isize;
                }
                while i >= data.len() as isize {
                    i -= in_period as isize;
                }
                if i < 0 {
                    panic!("too short data");
                }
                data[i as usize]
            };
            let d = i - in_center;
            let k = out_center + d;
            if k >= 0 && k < result.len() as isize {
                result[k as usize] += amplitude
                    * Self::hanning_window(
                        (i - (in_center - in_period as isize)) as usize,
                        in_period * 2,
                    );
            }
        }
    }

    fn psola(&self, data: &[f32], info: &PsolaInfo) -> (Vec<f32>, PsolaInfo) {
        if let Some(in_period) = Self::period(
            data,
            (50, 800),
            data.iter().fold(0.0, |a, b| f32::max(a * a, b * b)) / 4.0,
        ) {
            let mut result = vec![0.0; data.len()];
            let ratio = self.ratio;
            Self::unitary_ola(
                data,
                &mut result,
                info.last_in_center - data.len() as isize,
                info.in_period as usize,
                info.last_out_center - data.len() as isize,
            );

            let in_period = in_period as isize;
            let out_period = (in_period as f32 / ratio) as isize;
            let mut in_center = info.last_in_center - data.len() as isize + in_period;
            let mut out_center = info.last_out_center - data.len() as isize + out_period;
            while out_center < result.len() as isize {
                while (in_center + in_period - out_center).abs() < (in_center - out_center).abs()
                    && in_center + in_period < data.len() as isize
                {
                    in_center += in_period;
                }
                Self::unitary_ola(data, &mut result, in_center, in_period as usize, out_center);
                out_center += out_period;
            }

            Self::unitary_ola(data, &mut result, in_center, in_period as usize, out_center);
            let info = PsolaInfo {
                last_in_center: in_center,
                last_out_center: out_center - out_period,
                in_period: in_period as isize,
            };
            (result, info)
        } else {
            (data.to_vec(), info.clone())
        }
    }

    fn process_chunk(&mut self, chunk: DataChunk) -> Option<DataChunk> {
        let chunk = match chunk {
            DataChunk::Real(chunk) => chunk,
            _ => {
                eprintln!("incompatible input {}: {}", file!(), line!());
                return None;
            }
        };
        let channels = *chunk.metadata().channels();
        while self.psola_info.len() < channels {
            self.psola_info.push(PsolaInfo {
                last_out_center: 0,
                last_in_center: 0,
                in_period: 0,
            });
        }
        let (samples, info) = (0..channels)
            .zip(self.psola_info.iter())
            .map(|(c, info)| self.psola(chunk.samples(c), info))
            .unzip();
        let out_chunk = DataChunk::Real(GenericDataChunk::new(
            samples,
            chunk.metadata().clone(),
            *chunk.duration(),
            chunk.window_info().clone(),
        ));
        self.psola_info = info;
        Some(out_chunk)
    }
}
