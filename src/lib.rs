//! A crate implementing a Brukhard Keller tree datastructure which allows for fast querying of
//! "close" matches on discrete distances.
//!
//! ```rust
//! use bktree::*;
//!
//! let mut bk = BkTree::new(hamming_distance);
//! bk.insert_all(vec![0, 4, 5, 14, 15]);
//!
//! let (words, dists): (Vec<i32>, Vec<isize>) = bk.find(13, 1).into_iter().unzip();
//! assert_eq!(words, [5, 15]);
//! assert_eq!(dists, [1, 1]);
//! ```
//!
//! ```rust
//! use bktree::*;
//!
//! let mut bk = BkTree::new(levenshtein_distance);
//! bk.insert_all(vec![
//!     "book", "books", "boo", "boon", "cook", "cake", "cape", "cart",
//! ]);
//! let (words, dists): (Vec<&str>, Vec<isize>) = bk.find("bo", 2).into_iter().unzip();
//! assert_eq!(words, ["book", "boo", "boon"]);
//! assert_eq!(dists, [2, 1, 2]);
//! ```

/// Typical distance functions to use with the BK-tree
pub mod distance;

pub use distance::*;

#[derive(Debug, Eq, PartialEq)]
struct Node<T> {
    word: T,
    children: Vec<(isize, Node<T>)>,
}

/// A BK-tree datastructure
///
pub struct BkTree<T> {
    root: Option<Box<Node<T>>>,
    dist: Box<dyn Fn(&T, &T) -> isize>,
}

impl<T> BkTree<T> {
    /// Create a new BK-tree with a given distance function
    pub fn new(dist: impl Fn(&T, &T) -> isize + 'static) -> Self {
        Self {
            root: None,
            dist: Box::new(dist),
        }
    }

    /// Insert every element from a given iterator in the BK-tree
    pub fn insert_all<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for i in iter {
            self.insert(i);
        }
    }

    /// Insert a new element in the BK-tree
    pub fn insert(&mut self, val: T) {
        match self.root {
            None => {
                self.root = Some(Box::new(Node {
                    word: val,
                    children: Vec::new(),
                }))
            }
            Some(ref mut root_node) => {
                let mut u = &mut **root_node;
                loop {
                    let k = (self.dist)(&u.word, &val);
                    if k == 0 {
                        return;
                    }

                    let v = u.children.iter().position(|(dist, _)| *dist == k);
                    match v {
                        None => {
                            u.children.push((
                                k,
                                Node {
                                    word: val,
                                    children: Vec::new(),
                                },
                            ));
                            return;
                        }
                        Some(pos) => {
                            let (_, ref mut vnode) = u.children[pos];
                            u = vnode;
                        }
                    }
                }
            }
        }
    }

    /// Find the closest elements to a given value present in the BK-tree
    ///
    /// Returns pairs of element references and distances
    pub fn find(&self, val: T, max_dist: isize) -> Vec<(&T, isize)> {
        match self.root {
            None => Vec::new(),
            Some(ref root) => {
                let mut found = Vec::new();

                let mut candidates: std::collections::VecDeque<&Node<T>> =
                    std::collections::VecDeque::new();
                candidates.push_back(root);

                while let Some(n) = candidates.pop_front() {
                    let distance = (self.dist)(&n.word, &val);
                    if distance <= max_dist {
                        found.push((&n.word, distance));
                    }

                    candidates.extend(
                        n.children
                            .iter()
                            .filter(|(arc, _)| (*arc - distance).abs() <= max_dist)
                            .map(|(_, node)| node),
                    );
                }
                found
            }
        }
    }

    /// Convert the BK-tree into an iterator over its elements, in no particular order
    pub fn into_iter(self) -> IntoIter<T> {
        let mut queue = Vec::new();
        if let Some(root) = self.root {
            queue.push(*root);
        }
        IntoIter { queue }
    }
    /// Create an iterator over references of BK-tree elements, in no particular order
    pub fn iter(&self) -> Iter<T> {
        let mut queue = Vec::new();
        if let Some(ref root) = self.root {
            queue.push(&**root);
        }
        Iter { queue }
    }
}

impl<T> IntoIterator for BkTree<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/// Iterator over BK-tree elements
pub struct IntoIter<T> {
    queue: Vec<Node<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|node| {
            self.queue.extend(node.children.into_iter().map(|(_, n)| n));
            node.word
        })
    }
}

/// Iterator over BK-tree elements, by reference
pub struct Iter<'a, T> {
    queue: Vec<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|node| {
            self.queue.extend(node.children.iter().map(|(_, n)| n));
            &node.word
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::distance::*;
    use crate::BkTree;
    #[test]
    fn levenshtein_distance_test() {
        let mut bk = BkTree::new(levenshtein_distance);
        bk.insert_all(vec![
            "book", "books", "boo", "boon", "cook", "cake", "cape", "cart",
        ]);
        let (words, dists): (Vec<&str>, Vec<isize>) = bk.find("bo", 2).into_iter().unzip();
        assert_eq!(words, ["book", "boo", "boon"]);
        assert_eq!(dists, [2, 1, 2]);
    }

    #[test]
    fn hamming_distance_test() {
        let mut bk = BkTree::new(hamming_distance);
        bk.insert_all(vec![0, 4, 5, 14, 15]);

        let (words, dists): (Vec<i32>, Vec<isize>) = bk.find(13, 1).into_iter().unzip();
        assert_eq!(words, [5, 15]);
        assert_eq!(dists, [1, 1]);
    }

    #[test]
    fn iterators_test() {
        let mut bk = BkTree::new(hamming_distance);
        bk.insert_all(vec![0, 4, 5, 14, 15]);

        let iter_res: Vec<&i32> = bk.iter().collect();
        assert_eq!(iter_res, [&0, &15, &14, &5, &4]);
        let intoiter_res: Vec<i32> = bk.into_iter().collect();
        assert_eq!(intoiter_res, [0, 15, 14, 5, 4]);
    }
}
