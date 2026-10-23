pub struct Registry<T> {
    slots: Vec<Slot<T>>,
}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self { slots: Vec::new() }
    }
}

impl<T> Registry<T> {
    pub fn insert(&mut self, value: T) -> Option<u64> {
        if self.len() >= 16 {
            return None;
        }
        let index = self
            .slots
            .iter()
            .position(|slot| slot.value.is_none() && slot.generation < u32::MAX);
        let index = index.unwrap_or_else(|| {
            self.slots.push(Slot {
                generation: 1,
                value: None,
            });
            self.slots.len() - 1
        });
        let slot = &mut self.slots[index];
        slot.value = Some(value);
        Some((u64::from(slot.generation) << 32) | (index as u64 + 1))
    }

    pub fn get(&self, handle: u64) -> Option<&T> {
        let index = (handle as u32).checked_sub(1)? as usize;
        let slot = self.slots.get(index)?;
        (slot.generation == (handle >> 32) as u32)
            .then_some(slot.value.as_ref())
            .flatten()
    }

    pub fn remove(&mut self, handle: u64) -> Option<T> {
        let index = (handle as u32).checked_sub(1)? as usize;
        let slot = self.slots.get_mut(index)?;
        if slot.generation != (handle >> 32) as u32 {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation = slot.generation.saturating_add(1);
        Some(value)
    }

    pub fn len(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.value.is_some())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_handle_cannot_reach_reused_slot() {
        let mut registry = Registry::default();
        let first = registry.insert(42).unwrap();
        assert_eq!(registry.remove(first), Some(42));
        let second = registry.insert(7).unwrap();
        assert_ne!(first, second);
        assert_eq!(registry.get(first), None);
        assert_eq!(registry.remove(first), None);
        assert_eq!(registry.get(second), Some(&7));
        assert_eq!(registry.get(0), None);
        assert_eq!(registry.get(u64::MAX), None);
    }

    #[test]
    fn exhausted_generation_is_not_reused() {
        let mut registry = Registry {
            slots: vec![Slot {
                generation: u32::MAX,
                value: Some(1),
            }],
        };
        let handle = (u64::from(u32::MAX) << 32) | 1;
        assert_eq!(registry.remove(handle), Some(1));
        let next = registry.insert(2).unwrap();
        assert_eq!(next as u32, 2);
        assert_eq!(registry.get(handle), None);
    }

    #[test]
    fn instance_count_is_bounded() {
        let mut registry = Registry::default();
        for _ in 0..16 {
            assert!(registry.insert(()).is_some());
        }
        assert_eq!(registry.len(), 16);
        assert!(registry.insert(()).is_none());
    }
}
