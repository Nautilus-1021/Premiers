use std::{iter::repeat_n, sync::{LazyLock, RwLock, atomic::{AtomicU32, AtomicUsize, Ordering}}};

pub static PARALLELISM: LazyLock<usize> = LazyLock::new(|| { std::thread::available_parallelism().unwrap().into() });
//pub static PARALLELISM: LazyLock<usize> = LazyLock::new(|| 4);

pub struct AtomicVec {
    storage: RwLock<Box<[AtomicU32]>>,
    usage: AtomicUsize,
    alloc_size: AtomicUsize,
    extend_size: usize,
    realloc_stats: AtomicUsize
}

struct Leurre(AtomicU32);

impl Clone for Leurre {
    fn clone(&self) -> Self {
        Self(AtomicU32::new(self.0.load(Ordering::Relaxed)))
    }
}

const LT: Leurre = Leurre(AtomicU32::new(0));

fn alloc_atomic_vec(sz: usize) -> Box<[AtomicU32]> {
    repeat_n(LT, sz).map(|lr| lr.0).collect()
}

impl AtomicVec {
    pub fn new(size_estimate: usize, extend_size: usize) -> Self {
        let storage: RwLock<Box<[AtomicU32]>> = RwLock::new(alloc_atomic_vec(size_estimate));

        Self { storage, usage: AtomicUsize::new(0), alloc_size: AtomicUsize::new(size_estimate), extend_size, realloc_stats: AtomicUsize::new(0) }
    }

    pub fn push(&self, nmb: u32) -> Result<(), &'static str> {
        if self.usage.load(Ordering::SeqCst) + *PARALLELISM < self.alloc_size.load(Ordering::SeqCst) {
            let Ok(lock) = self.storage.read() else { return Err("Impossible d'acquérir le RwLock") };

            assert_eq!(lock.len(), self.alloc_size.load(Ordering::SeqCst), "Mismatch between advertised allocation and reality");

            let idx = self.usage.fetch_add(1, Ordering::SeqCst);

            let res = lock[idx].swap(nmb, Ordering::SeqCst);
            if res != 0 {
                return Err("Value was already initialized (no realloc)")
            }
        } else {
            self.realloc_stats.fetch_add(1, Ordering::Relaxed);
            let Ok(mut lock) = self.storage.write() else { return Err("Impossible d'acquérir le RwLock") };

            let idx = self.usage.fetch_add(1, Ordering::SeqCst);

            let res = lock[idx].swap(nmb, Ordering::SeqCst);
            if res != 0 {
                return Err("Value was already initialized (realloc)")
            }

            let prev_alloc_size = self.alloc_size.fetch_add(self.extend_size, Ordering::SeqCst);

            let new_storage = alloc_atomic_vec(prev_alloc_size + self.extend_size);

            for (new, old) in new_storage.iter().zip(lock.iter().take(self.usage.load(Ordering::SeqCst))) {
                new.store(old.load(Ordering::SeqCst), Ordering::SeqCst);
            }

            *lock = new_storage;
        }

        Ok(())
    }

    pub fn iter(&self) -> AtomicVecIter<'_> {
        AtomicVecIter { vec: self, idx: 0 }
    }

    pub fn get_realloc_nmb(&self) -> usize {
        self.realloc_stats.load(Ordering::Relaxed)
    }
}

pub struct AtomicVecIter<'lf> {
    vec: &'lf AtomicVec,
    idx: usize
}

impl<'lf> Iterator for AtomicVecIter<'lf> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.vec.usage.load(Ordering::Relaxed) {
            return None
        }

        let val = {
            let Ok(lock) = self.vec.storage.read() else { return None };
            lock[self.idx].load(Ordering::Relaxed)
        };

        self.idx += 1;

        Some(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let vec1 = AtomicVec::new(10, 1);

        for x in 3..16 {
            vec1.push(x).unwrap();
        }

        for (idx, el) in vec1.iter().enumerate() {
            assert_eq!(idx + 3, el as usize);
        }
    }
}