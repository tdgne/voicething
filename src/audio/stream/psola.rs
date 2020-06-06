use super::super::common::*;
use super::node::*;
use super::port::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::FFTplanner;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
struct PsolaInfo {
    last_in_center: isize,
    last_out_center: isize,
    in_period: isize,
}

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct PsolaNode {
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
    ratio: f32,
    #[serde(skip)]
    psola_info: Vec<PsolaInfo>,
    id: Uuid,
}

impl NodeTrait for PsolaNode {
    fn id(&self) -> Uuid {
        self.id
    }
    fn inputs(&self) -> &[InputPort] {
        &self.inputs
    }
    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut self.inputs
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }
    fn add_input(&mut self) -> Result<&mut InputPort, Box<dyn std::error::Error>> {
        if self.inputs.len() == 0 {
            self.inputs.push(InputPort::new(self.id));
            Ok(&mut self.inputs[0])
        } else {
            Err(Box::new(PortAdditionError))
        }
    }
    fn add_output(&mut self) -> Result<&mut OutputPort, Box<dyn std::error::Error>> {
        self.outputs.push(OutputPort::new(self.id));
        let l = self.outputs.len();
        Ok(&mut self.outputs[l - 1])
    }
    fn run_once(&mut self) {
        if self.inputs.len() != 1 {
            return;
        }
        if let Some(chunk) = self.inputs[0].try_recv().ok() {
            let chunk = self.process_chunk(chunk);
            for output in self.outputs().iter() {
                let _ = output.try_send(chunk.clone());
            }
        }
    }
}

impl PsolaNode {
    pub fn new(ratio: f32) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
            ratio,
            psola_info: vec![],
            id: Uuid::new_v4(),
        }
    }

    pub fn ratio(&mut self) -> f32 {
        self.ratio
    }

    pub fn ratio_mut(&mut self) -> &mut f32 {
        &mut self.ratio
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

    fn process_chunk(&mut self, chunk: SampleChunk) -> SampleChunk {
        let chunk = match chunk {
            SampleChunk::Real(chunk) => chunk,
            _ => panic!("Incompatible input"),
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
        let out_chunk = SampleChunk::Real(GenericSampleChunk::new(
            samples,
            chunk.metadata().clone(),
            *chunk.duration_samples(),
            chunk.window_info().clone(),
        ));
        self.psola_info = info;
        out_chunk
    }
}
