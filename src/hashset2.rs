use std::collections::hash_set::Iter;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub trait WithTwoHashes: Eq + Copy {
    fn hash1<H: Hasher>(&self, state: &mut H);
    fn hash2<H: Hasher>(&self, state: &mut H);
}

#[derive(Eq, PartialEq, Debug)]
struct Hash1Wrapper<T: WithTwoHashes>(T);
impl<T: WithTwoHashes> Hash for Hash1Wrapper<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash1(state);
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Hash2Wrapper<T: WithTwoHashes>(T);
impl<T: WithTwoHashes> Hash for Hash2Wrapper<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash2(state);
    }
}

pub struct HashSet2Iter<'a, T: WithTwoHashes>(Iter<'a, Hash1Wrapper<T>>);
impl<'a, T: WithTwoHashes> Iterator for HashSet2Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(value) => Some(&(*value).0),
            None => None,
        }
    }
}

pub struct HashSet2<T: WithTwoHashes>(HashSet<Hash1Wrapper<T>>, HashSet<Hash2Wrapper<T>>, u32);
impl<'a, T: WithTwoHashes> IntoIterator for &'a HashSet2<T> {
    type Item = &'a T;

    type IntoIter = HashSet2Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        HashSet2Iter(self.0.iter())
    }
}
impl<T: WithTwoHashes> HashSet2<T> {
    pub fn new() -> Self {
        HashSet2(HashSet::new(), HashSet::new(), 0)
    }

    pub fn len(&self) -> u32 {
        self.2
    }

    pub fn insert(&mut self, value: T) -> T {
        match self.get(value) {
            Some(result) => result,
            None => {
                self.0.insert(Hash1Wrapper(value));
                self.1.insert(Hash2Wrapper(value));
                self.2 += 1;
                value
            }
        }
    }

    pub fn get(&self, value: T) -> Option<T> {
        match self.0.get(&Hash1Wrapper(value)) {
            Some(result) => Some(result.0),
            None => match self.1.get(&Hash2Wrapper(value)) {
                Some(result) => Some(result.0),
                None => None,
            },
        }
    }

    pub fn contains(&self, value: T) -> bool {
        self.0.contains(&Hash1Wrapper(value)) || self.1.contains(&Hash2Wrapper(value))
    }

    pub fn slow_remove(&mut self, value: T) {
        self.0.retain(|x| x.0 != value);
        self.1.retain(|x| x.0 != value);
    }

    pub fn as_vector(&self) -> Vec<T> {
        let mut result = vec![];
        for Hash1Wrapper(value) in &self.0 {
            let mut found = false;
            for value1 in result.as_slice() {
                if *value1 == *value {
                    found = true;
                    break;
                }
            }
            if !found {
                result.push(*value);
            }
        }
        result
    }

    pub fn iter(&self) -> HashSet2Iter<T> {
        HashSet2Iter(self.0.iter())
    }
}

#[derive(Debug)]
pub struct HashMap2<T: WithTwoHashes, V: Copy>(
    HashMap<Hash1Wrapper<T>, V>,
    HashMap<Hash2Wrapper<T>, V>,
    u32,
);
impl<T: WithTwoHashes, V: Copy> HashMap2<T, V> {
    pub fn new() -> Self {
        HashMap2(HashMap::new(), HashMap::new(), 0)
    }

    pub fn insert_if_new(&mut self, key: T, value: V) {
        let current_value = self.get(key);
        match current_value {
            Some(_) => (),
            None => {
                self.0.insert(Hash1Wrapper(key), value);
                self.1.insert(Hash2Wrapper(key), value);
                self.2 += 1;
            }
        }
    }

    pub fn get(&self, key: T) -> Option<V> {
        match self.0.get(&Hash1Wrapper(key)) {
            Some(result) => Some(*result),
            None => match self.1.get(&Hash2Wrapper(key)) {
                Some(result) => Some(*result),
                None => None,
            },
        }
    }

    pub fn contains_key(&self, value: T) -> bool {
        self.0.contains_key(&Hash1Wrapper(value)) || self.1.contains_key(&Hash2Wrapper(value))
    }

    pub fn len(&self) -> u32 {
        self.2
    }
}

mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    #[allow(dead_code)]
    struct HT2(f64);
    impl PartialEq for HT2 {
        fn eq(&self, x: &HT2) -> bool {
            (x.0 - self.0).abs() < 1e-10
        }
    }
    impl Eq for HT2 {}
    impl WithTwoHashes for HT2 {
        fn hash1<H: Hasher>(&self, state: &mut H) {
            state.write_i32((self.0 * 1000.0) as i32);
        }
        fn hash2<H: Hasher>(&self, state: &mut H) {
            state.write_i32((self.0 * 1000.5) as i32);
        }
    }

    #[test]
    fn test_hash_set_insert_and_check() {
        let values = [HT2(0.2), HT2(0.3), HT2(0.19999999999999)];

        assert_eq!((values[0].0 * 1000.0) as i32, 200);
        assert_eq!((values[2].0 * 1000.0) as i32, 199);
        assert_eq!((values[0].0 * 1000.5) as i32, 200);
        assert_eq!((values[2].0 * 1000.5) as i32, 200);
        let mut hs: HashSet2<HT2> = HashSet2::new();
        let v0 = hs.insert(values[0]);
        assert!(v0.0 == values[0].0);
        assert!(hs.contains(values[2]));
        let v0a = hs.insert(values[0]);
        assert!(v0a.0 == values[0].0);
        assert_eq!(hs.0.len(), 1);
        assert_eq!(hs.1.len(), 1);
        let v2 = hs.insert(values[2]);
        assert!(v2.0 == values[0].0);
        assert_eq!(hs.0.len(), 1);
        assert_eq!(hs.1.len(), 1);
        hs.insert(values[1]);
        assert_eq!(hs.0.len(), 2);
        assert_eq!(hs.1.len(), 2);
    }

    #[test]
    fn test_hash_map_insert_and_check() {
        let values = [HT2(0.2), HT2(0.3), HT2(0.19999999999999)];

        assert_eq!((values[0].0 * 1000.0) as i32, 200);
        assert_eq!((values[2].0 * 1000.0) as i32, 199);
        assert_eq!((values[0].0 * 1000.5) as i32, 200);
        assert_eq!((values[2].0 * 1000.5) as i32, 200);
        let mut hm: HashMap2<HT2, i32> = HashMap2::new();
        hm.insert_if_new(values[0], 1);
        hm.insert_if_new(values[0], 2);
        assert_eq!(hm.0.len(), 1);
        assert_eq!(hm.1.len(), 1);
        hm.insert_if_new(values[2], 3);
        assert_eq!(hm.0.len(), 1);
        assert_eq!(hm.1.len(), 1);

        let v0_back = hm.get(values[0]);
        assert_eq!(v0_back, Some(1)); // This is incorrect, should be 3. Overriding values is not supported!

        let v2_back = hm.get(values[2]);
        assert_eq!(v2_back, Some(1));
    }
}
