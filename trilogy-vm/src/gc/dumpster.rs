use crate::cactus::RangeMap;
use std::{ops::DerefMut, sync::Mutex};

#[derive(Default)]
pub(crate) struct Dumpster {
    pub(super) trash: Mutex<RangeMap<usize>>,
}

impl Dumpster {
    pub fn trash_mut(&self) -> impl DerefMut<Target = RangeMap<usize>> + '_ {
        self.trash.lock().unwrap()
    }

    pub fn throw_out(&self, ranges: &RangeMap<bool>) {
        let mut trash = self.trash.lock().unwrap();
        for (range, _) in ranges.iter().filter(|(_, v)| *v) {
            trash.update(range, |x| {
                *x += 1;
            });
        }
    }
}
