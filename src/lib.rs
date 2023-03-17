//! A crate implementing a Brukhard Keller tree datastructure which allows for fast querying of
//! "close" matches on discrete distances.
//!
//! ```rust
//! use bktree::*;
//!
//! let mut bk = BkTree::new(HammingDistance);
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
//! let mut bk = BkTree::new(LevenshteinDistance);
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

#[cfg(feature = "serde-support")]
extern crate serde;

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde-support",
    derive(serde::Serialize, serde::Deserialize)
)]
struct Node<T> {
    word: T,
    children: Vec<(isize, Node<T>)>,
}

/// A BK-tree datastructure
///
#[cfg_attr(
    feature = "serde-support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct BkTree<T, D = distance::LevenshteinDistance> {
    root: Option<Node<T>>,
    dist: D,
}

impl<T, D> BkTree<T, D>
where
    D: Distance<T>,
{
    /// Create a new BK-tree with a given distance function
    pub fn new(dist: D) -> BkTree<T, D> {
        Self {
            root: None,
            dist,
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
                self.root = Some(Node {
                    word: val,
                    children: Vec::new(),
                })
            }
            Some(ref mut root_node) => {
                let mut u = root_node;
                loop {
                    let k = self.dist.distance(&u.word, &val);
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
                    let distance = self.dist.distance(&n.word, &val);
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
    /// Create an iterator over references of BK-tree elements, in no particular order
    pub fn iter(&self) -> Iter<T> {
        let mut queue = Vec::new();
        if let Some(ref root) = self.root {
            queue.push(root);
        }
        Iter { queue }
    }
}

impl<T, D> IntoIterator for BkTree<T, D> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut queue = Vec::new();
        if let Some(root) = self.root {
            queue.push(root);
        }
        IntoIter { queue }
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
    extern crate bincode;

    use crate::distance::*;
    use crate::BkTree;
    use std::fmt::Debug;
    #[test]
    fn levenshtein_distance_test() {
        let mut bk = BkTree::new(LevenshteinDistance);
        bk.insert_all(vec![
            "book", "books", "boo", "boon", "cook", "cake", "cape", "cart",
        ]);
        let (words, dists): (Vec<&str>, Vec<isize>) = bk.find("bo", 2).into_iter().unzip();
        assert_eq!(words, ["book", "boo", "boon"]);
        assert_eq!(dists, [2, 1, 2]);
    }

    #[test]
    fn hamming_distance_test() {
        let mut bk = BkTree::new(HammingDistance);
        bk.insert_all(vec![0, 4, 5, 14, 15]);

        let (words, dists): (Vec<i32>, Vec<isize>) = bk.find(13, 1).into_iter().unzip();
        assert_eq!(words, [5, 15]);
        assert_eq!(dists, [1, 1]);
    }

    #[test]
    fn iterators_test() {
        let mut bk = BkTree::new(HammingDistance);
        bk.insert_all(vec![0, 4, 5, 14, 15]);

        let iter_res: Vec<&i32> = bk.iter().collect();
        assert_eq!(iter_res, [&0, &15, &14, &5, &4]);
        let intoiter_res: Vec<i32> = bk.into_iter().collect();
        assert_eq!(intoiter_res, [0, 15, 14, 5, 4]);
    }

    #[cfg(feature = "serde-support")]
    #[test]
    fn test_serialization() {
        let mut bk: BkTree<&str> = BkTree::new(LevenshteinDistance);
        let words = vec![
            "book", "books", "boo", "boon", "cook", "cake", "cape", "cart",
        ];
        bk.insert_all(words.clone());

        // Test exact search (zero tolerance)
        for word in &words {
            let (word_list, dist_list): (Vec<&str>, Vec<isize>) =
                bk.find(word, 0).into_iter().unzip();
            assert_eq!(word_list, vec![*word]);
            assert_eq!(dist_list, vec![0]);
        }

        // Test fuzzy search
        let (word_list, dist_list): (Vec<&str>, Vec<isize>) = bk.find("ca", 3).into_iter().unzip();
        assert_eq!(word_list, vec!["cake", "boo", "cape", "cart", "cook"]);
        assert_eq!(dist_list, vec![2, 3, 2, 2, 3]);

        // Test for false positives
        let (word_list, dist_list): (Vec<&str>, Vec<isize>) =
            bk.find("not here", 0).into_iter().unzip();
        assert_eq!(word_list, vec![""; 0]);
        assert_eq!(dist_list, vec![0; 0]);

        let encoded_bk: Vec<u8> = bincode::serialize(&bk).unwrap();
        let decoded_bk: BkTree<&str> = bincode::deserialize(&encoded_bk[..]).unwrap();

        // Test exact search (zero tolerance)
        for word in &words {
            let (word_list, dist_list): (Vec<&str>, Vec<isize>) =
                decoded_bk.find(word, 0).into_iter().unzip();
            assert_eq!(word_list, vec![*word]);
            assert_eq!(dist_list, vec![0]);
        }

        // Test fuzzy search
        let (word_list, dist_list): (Vec<&str>, Vec<isize>) =
            decoded_bk.find("ca", 3).into_iter().unzip();
        assert_eq!(word_list, vec!["cake", "boo", "cape", "cart", "cook"]);
        assert_eq!(dist_list, vec![2, 3, 2, 2, 3]);

        // Test for false positives
        let (word_list, dist_list): (Vec<&str>, Vec<isize>) =
            decoded_bk.find("not here", 0).into_iter().unzip();
        assert_eq!(word_list, vec![""; 0]);
        assert_eq!(dist_list, vec![0; 0]);
    }
}
