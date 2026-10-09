//! A short list kept inline, with a fixed most it can hold — for the
//! per-person lists (memories, known events), so 5,000 people don't each
//! carry a heap allocation and its spare room. Reads like a slice; saved as
//! just the items it holds.

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Few<T: Copy + Default, const N: usize> {
    n: u8,
    items: [T; N],
}

impl<T: Copy + Default, const N: usize> Default for Few<T, N> {
    fn default() -> Self {
        Few { n: 0, items: [T::default(); N] }
    }
}

impl<T: Copy + Default, const N: usize> Few<T, N> {
    pub const fn new_with(fill: T) -> Self {
        Few { n: 0, items: [fill; N] }
    }

    /// Add one; returns false (and adds nothing) when full.
    pub fn push(&mut self, x: T) -> bool {
        if (self.n as usize) < N {
            self.items[self.n as usize] = x;
            self.n += 1;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, i: usize) -> T {
        let x = self.items[i];
        let n = self.n as usize;
        self.items.copy_within(i + 1..n, i);
        self.n -= 1;
        x
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        let mut j = 0;
        for i in 0..self.n as usize {
            if keep(&self.items[i]) {
                self.items[j] = self.items[i];
                j += 1;
            }
        }
        self.n = j as u8;
    }

    pub const fn capacity(&self) -> usize {
        N
    }
}

impl<T: Copy + Default, const N: usize> std::ops::Deref for Few<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.items[..self.n as usize]
    }
}

impl<T: Copy + Default, const N: usize> std::ops::DerefMut for Few<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.items[..self.n as usize]
    }
}

impl<T: Copy + Default + Serialize, const N: usize> Serialize for Few<T, N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.n as usize))?;
        for x in self.iter() {
            seq.serialize_element(x)?;
        }
        seq.end()
    }
}

impl<'de, T: Copy + Default + Deserialize<'de>, const N: usize> Deserialize<'de> for Few<T, N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v: Vec<T> = Vec::deserialize(d)?;
        let mut f = Few::default();
        for x in v.into_iter().take(N) {
            f.push(x);
        }
        Ok(f)
    }
}
