#[cfg(test)]
extern crate rand;
#[cfg(test)]
use rand::Rng;

use std::collections::VecDeque;
use std::collections::vec_deque::Iter as VecDequeIter;
use std::collections::vec_deque::IterMut as VecDequeIterMut;

#[derive(Debug)]
enum Entry<V> {
    Empty,
    Full(V)
}

#[derive(Debug)]
pub struct ConsecVecMap<V> {
    head: Option<isize>,
    deque: VecDeque<Entry<V>>,
    len: usize,
}

pub struct Iter<'a, V: 'a> {
    internal: VecDequeIter<'a, Entry<V>>,
    iteration: isize,
}

pub struct IterMut<'a, V: 'a> {
    internal: VecDequeIterMut<'a, Entry<V>>,
    iteration: isize,
}

impl <V> Entry<V> {
    pub fn is_empty(&self) -> bool {
        match self {
            &Entry::Empty => true,
            &Entry::Full(_) => false
        }
    }

    pub fn is_full(&self) -> bool {
        !self.is_empty()
    }

    pub fn to_option(self) -> Option<V> {
        match self {
            Entry::Empty => None,
            Entry::Full(v) => Some(v)
        }
    }
}

impl <V> ConsecVecMap<V> {
    /// Constructs a new empty ConsecVecMap
    pub fn new() -> ConsecVecMap<V> {
        ConsecVecMap {
            head: None,
            deque: VecDeque::new(),
            len: 0
        }
    }

    /// Constructs a new empty ConsecVecMap with a given capacity
    pub fn with_capacity(capacity: usize) -> ConsecVecMap<V> {
        ConsecVecMap {
            head: None,
            deque: VecDeque::with_capacity(capacity),
            len: 0
        }
    }

    /// Checks to see if the map is empty
    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }

    /// Returns the number of items that are inside the map
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, key: isize) -> Option<&V> {
        if let Some(head) = self.head {
            if key < head || key >= head + self.deque.len() as isize {
                None
            } else {
                match &self.deque[(key - head) as usize] {
                    &Entry::Empty => None,
                    &Entry::Full(ref value) => Some(value)
                }
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: isize) -> Option<&mut V> {
        unsafe { std::mem::transmute(self.get(key)) }
    }

    /// Inserts a new key-value pair into the map
    pub fn insert(&mut self, key: isize, value: V) -> Option<V> {
        use std::mem::swap;
        if let Some(head) = self.head {
            if key < head {
                let diff = head - key;
                for _ in 0 .. diff - 1 {
                    self.deque.push_front(Entry::Empty);
                }
                self.deque.push_front(Entry::Full(value));
                self.head = Some(key);
                self.len += 1;
                None
            } else if key < head + self.deque.len() as isize {
                let mut filled = Entry::Full(value);
                swap(&mut filled, &mut self.deque[(key - head) as usize]);
                if filled.is_empty() {
                    self.len += 1;
                }
                filled.to_option()
            } else {
                let diff = key - (head + self.deque.len() as isize);
                for _ in 0 .. diff {
                    self.deque.push_back(Entry::Empty);
                }
                self.deque.push_back(Entry::Full(value));
                self.len += 1;
                None
            }
        } else {
            debug_assert!(self.deque.is_empty());
            debug_assert!(self.len == 0);
            self.deque.push_back(Entry::Full(value));
            self.len = 1;
            self.head = Some(key);
            None
        }
    }

    /// Tries to remove the object with the associated key
    pub fn remove(&mut self, key: isize) -> Option<V> {
        use std::mem::swap;
        if let Some(head) = self.head {
            if key < head || key >= head + self.deque.len() as isize {
                None
            } else {
                let mut slot = Entry::Empty;
                swap(&mut slot, &mut self.deque[(key - head) as usize]);
                self.maintain();
                if slot.is_full() {
                    self.len -= 1;
                }
                slot.to_option()
            }
        } else {
            None
        }
    }

    /// Checks to see if this map contains a key
    pub fn contains_key(&mut self, key: isize) -> bool {
        if let Some(head) = self.head {
            if key < head || key >= head + self.deque.len() as isize {
                false
            } else {
                self.deque[(key - head) as usize].is_full()
            }
        } else {
            false
        }
    }

    /// Returns an iterator over (isize, &V)
    pub fn iter(&self) -> Iter<V> {
        Iter {
            internal: self.deque.iter(),
            iteration: self.head.unwrap_or(0)
        }
    }

    /// Returns an iterator over (isize, &mut V)
    pub fn iter_mut(&mut self) -> IterMut<V> {
        IterMut {
            internal: self.deque.iter_mut(),
            iteration: self.head.unwrap_or(0)
        }
    }

    fn maintain(&mut self) {
        for _ in 0 .. self.deque.len() {
            if self.deque[0].is_empty() {
                self.deque.pop_front();
                if let Some(head) = self.head.as_mut() {
                    *head += 1;
                }
            } else if self.deque[self.deque.len() - 1].is_empty() {
                self.deque.pop_back();
            } else {
                break;
            }
        }
        if self.deque.is_empty() {
            self.head == None;
        } else {
        }
    }
}

impl <'a, V> Iterator for Iter<'a, V> {
    type Item = (isize, &'a V);

    fn next(&mut self) -> Option<(isize, &'a V)> {
        match self.internal.next() {
            Some(&Entry::Empty) => {
                self.iteration += 1;
                self.next()
            }
            Some(&Entry::Full(ref v)) => {
                let r = (self.iteration, v);
                self.iteration += 1;
                Some(r)
            }
            None => None
        }
    }
}

impl <'a, V> Iterator for IterMut<'a, V> {
    type Item = (isize, &'a mut V);

    fn next(&mut self) -> Option<(isize, &'a mut V)> {
        match self.internal.next() {
            Some(&mut Entry::Empty) => {
                self.iteration += 1;
                self.next()
            }
            Some(&mut Entry::Full(ref mut v)) => {
                let r = (self.iteration, v);
                self.iteration += 1;
                Some(r)
            }
            None => None
        }
    }
}

#[test]
fn single_insert() {
    let mut map = ConsecVecMap::new();
    map.insert(0, 10);
    assert!(!map.is_empty());
    assert!(map.len() == 1);
    assert!(map.contains_key(0));
    assert!(map.get(0) == Some(&10));
    assert!(map.get_mut(0) == Some(&mut 10));
    assert!(map.remove(0) == Some(10));
    assert!(map.is_empty());
    assert!(map.len() == 0);
}

#[test]
fn multi_insert() {
    for x in 0 .. 100 {
        let mut map = ConsecVecMap::new();
        let mut count = 0;
        for i in (x + 1) .. (x + 10) {
            let mut k = i * i;
            count += 1;

            assert!(map.insert(i, k).is_none());
            assert!(!map.is_empty());
            assert_eq!(map.len(), count);
            assert!(map.contains_key(i));
            assert!(map.get(i) == Some(&k));
            assert!(map.get_mut(i) == Some(&mut k));
        }

        for i in (x + 1) .. (x + 10) {
            let k = i * i;
            count -= 1;

            assert!(map.remove(i) == Some(k));
            println!("{:#?}", map);
            assert!(!map.contains_key(i));
            assert!(map.get(i).is_none());
            assert_eq!(map.len(), count);
        }

        assert!(map.is_empty());
        assert!(map.len() == 0);
    }
}

#[test]
fn regression() {
    let mut map = ConsecVecMap::new();
    assert!(map.insert(0, ()).is_none());
    assert!(map.insert(75, ()).is_none());
    println!("{:?}", map);
    println!("{:?}", map.len());
    println!("{:?}", map.deque.len());
    assert!(map.insert(74, ()).is_none());
}

#[test]
fn fuzz() {
    for _ in 0 .. 100 {
        println!("============");
        let mut random_vec: Vec<_> = (-100 .. 100).collect();
        let mut map = ConsecVecMap::new();
        let mut iter = 0;
        rand::thread_rng().shuffle(&mut random_vec);

        for i in random_vec.iter().cloned() {
            let i_3 = i * i * i;
            iter += 1;

            assert_eq!(map.insert(i, i_3), None);
            assert!(!map.is_empty());
            assert_eq!(map.len(), iter);
        }

        assert_eq!(map.deque.len(), iter);

        for i in random_vec.iter().cloned() {
            let i_3 = i * i * i;
            iter -= 1;

            assert_eq!(map.remove(i), Some(i_3));
            assert_eq!(map.len(), iter);
        }
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }
}
