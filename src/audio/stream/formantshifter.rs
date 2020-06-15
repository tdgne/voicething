use super::super::common::*;
use super::node::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::num_traits::*;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Shift {
    pub from: f32,
    pub to: f32,
}

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct FormantShifter {
    io: NodeIo,
    id: NodeId,
    shifts: Vec<Shift>,
    #[serde(skip)]
    prev_unwrapped_phases: Vec<Vec<f32>>,
    #[serde(skip)]
    prev_envelope: Vec<f32>,
    #[serde(skip)]
    prev_delta_f: f32,
    #[serde(skip)]
    prev_duration: Option<usize>,
}

impl HasNodeIo for FormantShifter {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl FormantShifter {
    pub fn new() -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            prev_unwrapped_phases: vec![],
            shifts: vec![],
            prev_envelope: vec![],
            prev_delta_f: 1.0,
            prev_duration: None,
        }
    }

    pub fn shifts(&self) -> &[Shift] {
        &self.shifts
    }
    pub fn shifts_mut(&mut self) -> &mut Vec<Shift> {
        &mut self.shifts
    }

    pub fn prev_envelope(&self) -> &[f32] {
        &self.prev_envelope
    }
    pub fn prev_delta_f(&self) -> f32 {
        self.prev_delta_f
    }

    pub fn prev_duration(&self) -> Option<usize> {
        self.prev_duration
    }

    pub fn process_chunk(&mut self, chunk: DataChunk) -> Option<DataChunk> {
        let channels = *chunk.metadata().channels();
        while self.prev_unwrapped_phases.len() < channels {
            self.prev_unwrapped_phases.push(vec![]);
        }
        for p in self.prev_unwrapped_phases.iter_mut() {
            while p.len() < *chunk.duration() {
                p.push(0.0);
            }
        }

        let mut incompatible = false;
        let samples = (0..channels)
            .map(|c| match &chunk {
                DataChunk::Real(_) => {
                    eprintln!("incompatible input {}: {}", file!(), line!());
                    incompatible = true;
                    vec![]
                }
                DataChunk::Complex(chunk) => chunk.samples(c).to_vec(),
            })
            .collect::<Vec<_>>();
        if incompatible {
            return None;
        }

        let mut unwrapped_phases = vec![vec![]; channels];
        for c in 0..channels {
            let duration = samples[c].len();
            for i in 0..duration {
                let mut prev_unwrapped_phase = self.prev_unwrapped_phases[c][i];
                if prev_unwrapped_phase.is_nan() {
                    prev_unwrapped_phase = 0.0;
                }
                let pi = 3.141592;
                let phase = samples[c][i].arg() % (2.0 * pi);
                let prev_phase = prev_unwrapped_phase % (2.0 * pi);
                let unwrapped_phase = prev_unwrapped_phase + phase - prev_phase
                    + if phase - prev_phase < -pi {
                        2.0 * pi
                    } else if phase - prev_phase > pi {
                        -2.0 * pi
                    } else {
                        0.0
                    };
                unwrapped_phases[c].push(unwrapped_phase);
            }
        }

        self.prev_envelope = samples[0].iter().map(|s| s.norm()).collect::<Vec<_>>();
        let duration = *chunk.duration();
        let d_f = *chunk.metadata().sample_rate() as f32 / duration as f32;
        self.prev_delta_f = d_f;
        self.prev_duration = Some(*chunk.duration());

        let mut scaled = vec![vec![]; channels];
        for c in 0..channels {
            for i in 0..duration / 2 {
                let to_freq = i as f32 * d_f;
                let shifts = self.closest_shift_triplet_by_to_freq(to_freq);
                let from_freq = if to_freq == shifts[1].to {
                    to_freq
                } else if to_freq < shifts[1].to {
                    (to_freq - shifts[0].to) / (shifts[1].to - shifts[0].to)
                        * (shifts[1].from - shifts[0].from)
                        + shifts[0].from
                } else {
                    (to_freq - shifts[1].to) / (shifts[2].to - shifts[1].to)
                        * (shifts[2].from - shifts[1].from)
                        + shifts[1].from
                };
                let from_i = (from_freq / d_f).round() as usize;
                if from_i < duration / 2 && from_i >= 0 {
                    scaled[c].push(samples[c][from_i]);
                } else {
                    scaled[c].push(Complex32::zero());
                }
            }
            for i in duration / 2..duration {
                let mirrored_index = duration - i - 1;
                let mirrored_sample = scaled[c][mirrored_index].conj();
                scaled[c].push(mirrored_sample);
            }
        }

        self.prev_unwrapped_phases = unwrapped_phases;

        let new_chunk = GenericDataChunk::new(
            scaled,
            chunk.metadata().clone(),
            chunk.duration().clone(),
            chunk.window_info().clone(),
        );
        Some(DataChunk::Complex(new_chunk))
    }

    pub fn add_shift(&mut self, shift: Shift) {
        self.shifts.push(shift);
        self.shifts
            .sort_by(|a, b| a.from.partial_cmp(&b.from).unwrap());
    }

    fn closest_shift_triplet_by_to_freq(&self, f: f32) -> Vec<Shift> {
        let prepend = Shift { from: 0.0, to: 0.0 };
        let append = Shift {
            from: 20000.0,
            to: 20000.0,
        };
        let mut shifts = if self.shifts.len() == 0 {
            return vec![prepend, Shift { from: f, to: f }, append];
        } else {
            let mut shifts = self.shifts.clone();
            shifts.insert(0, prepend);
            shifts.push(append);
            shifts
        };
        let mut closest_i = None;
        let mut closest_to_freq: Option<f32> = None;
        for (i, shift) in shifts.iter().enumerate().skip(1).take(self.shifts.len()) {
            if let Some(mut closest_to_freq) = closest_to_freq {
                if (closest_to_freq - f).abs() > (shift.to - f).abs() {
                    closest_to_freq = shift.to;
                    closest_i = Some(i);
                }
            } else {
                closest_to_freq = Some(shift.to);
                closest_i = Some(i);
            }
        }
        shifts[(closest_i.unwrap() - 1)..(closest_i.unwrap() + 2)].to_vec()
    }
}

impl NodeTrait for FormantShifter {
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
                    let result = output.try_send(chunk.clone());
                }
            }
        }
    }
}
