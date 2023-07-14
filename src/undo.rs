use std::time::Instant;

use itertools::{EitherOrBoth::*, Itertools};
use zstd::bulk;

pub trait Undo: Clone {
    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn size_bytes(&self) -> usize;
}

#[derive(Debug)]
pub struct UndoableData<Data: Undo> {
    data:            Data,
    stack:           UndoStack,
    max_size_so_far: usize,
}

#[derive(Debug, Default)]
pub struct UndoStack {
    step_stack:  Vec<UndoStep>,
    step_number: usize,
}

#[derive(Debug, Default)]
pub struct UndoStep {
    compressed_delta: Vec<u8>,
    delta_size:       usize,
    old_size:         usize,
    new_size:         usize,
}

impl<Data: Undo> UndoableData<Data> {
    pub fn new(data: Data) -> Self {
        let max_size_so_far = data.size_bytes();
        Self { data, stack: UndoStack::default(), max_size_so_far }
    }

    pub fn write<F, R>(&mut self, writer: F) -> R
    where
        F: FnOnce(&mut Data) -> R,
    {
        let old_data = self.data.clone();
        let result = writer(&mut self.data);
        let step = UndoStep::delta(&old_data.to_bytes(), &self.data.to_bytes());
        self.stack.push(step);
        self.max_size_so_far = self.max_size_so_far.max(self.data.size_bytes());
        result
    }

    pub fn read<F, R>(&self, reader: F) -> R
    where
        F: FnOnce(&Data) -> R,
    {
        reader(&self.data)
    }

    pub fn can_undo(&self) -> bool {
        self.stack.step_number > 0
    }

    pub fn can_redo(&self) -> bool {
        self.stack.step_number < self.stack.step_stack.len()
    }

    pub fn undo(&mut self) {
        if let Some(step) = self.stack.undo() {
            let old_data_bytes = self.data.to_bytes();
            let mut new_data_bytes = step.apply_delta(&old_data_bytes);
            new_data_bytes.resize(step.old_size, 0);
            self.data = Data::from_bytes(new_data_bytes);
        }
    }

    pub fn redo(&mut self) {
        if let Some(step) = self.stack.redo() {
            let old_data_bytes = self.data.to_bytes();
            let mut new_data_bytes = step.apply_delta(&old_data_bytes);
            new_data_bytes.resize(step.new_size, 0);
            self.data = Data::from_bytes(new_data_bytes);
        }
    }

    pub fn clear_stack(&mut self) {
        self.stack.step_stack.clear();
        self.stack.step_number = 0;
    }
}

impl UndoStack {
    pub fn push(&mut self, step: UndoStep) {
        self.step_stack.truncate(self.step_number);
        self.step_number += 1;
        self.step_stack.push(step);
    }

    pub fn undo(&mut self) -> Option<&UndoStep> {
        (self.step_number > 0).then(|| {
            self.step_number -= 1;
            log::info!("Undo: {}", self.step_number);
            &self.step_stack[self.step_number]
        })
    }

    pub fn redo(&mut self) -> Option<&UndoStep> {
        (self.step_number < self.step_stack.len()).then(|| {
            self.step_number += 1;
            log::info!("Redo: {}", self.step_number);
            &self.step_stack[self.step_number - 1]
        })
    }
}

impl UndoStep {
    pub fn delta(old: &[u8], new: &[u8]) -> UndoStep {
        let time = Instant::now();
        let delta = old
            .iter()
            .zip_longest(new.iter())
            .map(|zipped| match zipped {
                Both(x, y) => *x ^ *y,
                Left(x) | Right(x) => *x,
            })
            .collect_vec();
        let compressed_delta = bulk::compress(&delta, 3).unwrap();
        log::info!("{:.3}ms, {:.1}kB", time.elapsed().as_secs_f64() * 1000.0, compressed_delta.len() as f64 / 1024.0);
        Self { compressed_delta, delta_size: delta.len(), old_size: old.len(), new_size: new.len() }
    }

    pub fn apply_delta(&self, from: &[u8]) -> Vec<u8> {
        let mut to = bulk::decompress(&self.compressed_delta, self.delta_size).unwrap();
        for zipped in to.iter_mut().zip_longest(from.iter()) {
            match zipped {
                Both(x, y) => *x ^= *y,
                _ => break,
            }
        }
        to
    }
}
