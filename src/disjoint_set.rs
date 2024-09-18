#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    Root { size: usize },
    Link { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointSet {
    entries: Vec<Entry>,
    num_sets: usize,
}

impl DisjointSet {
    pub fn new(len: usize) -> Self {
        Self {
            entries: vec![Entry::Root { size: 1 }; len],
            num_sets: len,
        }
    }

    pub fn values(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.len()).map(|i| self.find(i))
    }

    pub fn num_sets(&self) -> usize {
        self.num_sets
    }

    pub fn num_members(&self, index: usize) -> usize {
        match self.entries[self.find(index)] {
            Entry::Root { size } => size,
            Entry::Link { index: _ } => unreachable!(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn find(&self, index: usize) -> usize {
        let mut i = index;
        loop {
            match self.entries[i] {
                Entry::Root { size: _ } => break i,
                Entry::Link { index: j } => i = j,
            }
        }
    }

    fn compress(&mut self, index: usize, root: usize) {
        let mut i = index;
        loop {
            match self.entries[i] {
                Entry::Root { size: _ } => break,
                Entry::Link { index: j } => {
                    self.entries[i] = Entry::Link { index: root };
                    i = j;
                }
            }
        }
    }

    pub fn join(&mut self, mut a: usize, mut b: usize) {
        let mut a_root = self.find(a);
        let mut b_root = self.find(b);
        if a_root == b_root {
            return;
        }
        let a_size = self.num_members(a_root);
        let b_size = self.num_members(b_root);

        if a_size < b_size {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut a_root, &mut b_root);
        }

        match self.entries[a_root] {
            Entry::Root { size: _ } => {
                self.entries[b_root] = Entry::Link { index: a_root };
                self.entries[a_root] = Entry::Root {
                    size: a_size + b_size,
                };
                self.num_sets -= 1;
            }
            Entry::Link { index: _ } => unreachable!("a should be root"),
        }

        self.compress(a, a_root);
        self.compress(b, a_root);
    }

    pub fn is_singleton(&self, index: usize) -> bool {
        matches!(self.entries[index], Entry::Root { size: 1 })
    }

    pub fn depth(&self, index: usize) -> usize {
        let mut i = index;
        let mut depth = 0;
        loop {
            match self.entries[i] {
                Entry::Root { size: _ } => break depth,
                Entry::Link { index: j } => {
                    i = j;
                    depth += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let ds = DisjointSet::new(2);
        assert_eq!(ds.num_sets(), 2);
        assert_eq!(ds.find(0), 0);
        assert_eq!(ds.find(1), 1);
        assert!(ds.find(0) != ds.find(1));
        assert!(ds.is_singleton(0));
        assert!(ds.is_singleton(1));
    }

    #[test]
    fn test_root_of() {
        let ds = DisjointSet {
            entries: vec![
                Entry::Root { size: 4 },
                Entry::Link { index: 0 },
                Entry::Link { index: 1 },
                Entry::Link { index: 0 },
                Entry::Root { size: 1 },
            ],
            num_sets: 2,
        };

        assert_eq!(ds.find(0), 0);
        assert_eq!(ds.find(1), 0);
        assert_eq!(ds.find(2), 0);
        assert_eq!(ds.find(3), 0);
        assert_eq!(ds.find(4), 4);
    }

    #[test]
    fn test_join() {
        let mut ds = DisjointSet::new(4);
        ds.join(0, 1);
        println!("{:?}", ds);
        ds.join(2, 3);
        println!("{:?}", ds);

        assert!(ds.find(0) == ds.find(1));
        assert!(ds.find(1) != ds.find(2));
        assert!(ds.find(2) == ds.find(3));
        assert!(ds.find(3) != ds.find(0));
        assert!(ds.find(0) != ds.find(2));
        assert!(ds.find(1) != ds.find(3));
        assert_eq!(ds.num_sets(), 2);
        assert_eq!(ds.num_members(0), 2);
        assert_eq!(ds.num_members(1), 2);
        assert_eq!(ds.num_members(2), 2);
        assert_eq!(ds.num_members(3), 2);

        ds.join(3, 1);
        assert!(ds.find(0) == ds.find(1));
        assert!(ds.find(1) == ds.find(2));
        assert!(ds.find(2) == ds.find(3));
        assert!(ds.find(3) == ds.find(0));
        assert!(ds.find(0) == ds.find(2));
        assert!(ds.find(1) == ds.find(3));
        assert_eq!(ds.num_sets(), 1);
        assert_eq!(ds.num_members(0), 4);
        assert_eq!(ds.num_members(1), 4);
        assert_eq!(ds.num_members(2), 4);
        assert_eq!(ds.num_members(3), 4);

        println!("{:?}", ds);
        for i in 0..ds.len() {
            println!("{}: {}", i, ds.depth(i));
        }
    }

    #[test]
    fn test_join2() {
        let mut ds = DisjointSet::new(8);
        ds.join(0, 1);
        ds.join(2, 3);
        ds.join(4, 5);
        ds.join(6, 7);
        ds.join(1, 2);
        ds.join(4, 6);
        ds.join(0, 4);

        for i in 0..ds.len() {
            print!("{} ", ds.depth(i));
        }
        println!();
    }
}
