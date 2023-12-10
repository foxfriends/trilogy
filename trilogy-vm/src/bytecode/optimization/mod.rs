use super::chunk::Line;
use crate::{Annotation, Offset};
use std::{collections::HashSet, marker::PhantomData};

mod remove_noops;

use remove_noops::*;

pub(crate) struct LineAdjuster {
    annotations: Vec<Annotation>,
    lines: Vec<Line>,
    erased: HashSet<usize>,
}

impl LineAdjuster {
    pub fn new(lines: Vec<Line>, annotations: Vec<Annotation>) -> Self {
        Self {
            annotations,
            lines,
            erased: HashSet::default(),
        }
    }

    pub fn finish(mut self) -> (Vec<Line>, Vec<Annotation>) {
        let lines = self
            .lines
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !self.erased.contains(i))
            .map(|(_, l)| l)
            .collect();
        for annotation in &mut self.annotations.iter_mut() {
            let before = self
                .erased
                .iter()
                .filter(|&&i| i < annotation.start as usize)
                .count();
            let in_range = self
                .erased
                .iter()
                .filter(|&&i| i >= annotation.start as usize && i < annotation.end as usize)
                .count();
            annotation.start -= before as u32;
            annotation.end -= in_range as u32 + before as u32;
        }
        (lines, self.annotations)
    }
}

pub(crate) struct LineAdjusterEntries<'a> {
    adjuster: *mut LineAdjuster,
    index: usize,
    _pd: PhantomData<&'a ()>,
}

pub(crate) struct LineAdjusterEntry<'a> {
    adjuster: *mut LineAdjuster,
    index: usize,
    _pd: PhantomData<&'a ()>,
}

impl LineAdjusterEntry<'_> {
    fn as_line(&self) -> &Line {
        unsafe { &(*self.adjuster).lines[self.index] }
    }

    fn erase(&mut self) {
        unsafe {
            (*self.adjuster).erased.insert(self.index);
        }
    }
}

impl<'a> Iterator for LineAdjusterEntries<'a> {
    type Item = LineAdjusterEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let adjuster = unsafe { &mut *self.adjuster };

        while adjuster.erased.contains(&self.index) {
            self.index += 1;
        }
        if self.index == adjuster.lines.len() {
            return None;
        }

        let entry = LineAdjusterEntry {
            adjuster: self.adjuster,
            index: self.index,
            _pd: PhantomData,
        };
        self.index += 1;
        Some(entry)
    }
}

impl<'a> IntoIterator for &'a mut LineAdjuster {
    type IntoIter = LineAdjusterEntries<'a>;
    type Item = LineAdjusterEntry<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LineAdjusterEntries {
            adjuster: self,
            index: 0,
            _pd: PhantomData,
        }
    }
}

pub(super) fn optimize(
    lines: &mut LineAdjuster,
    _entrypoint: Option<Offset>,
    _force_reachable: &[String],
) {
    remove_noops(lines);
}
