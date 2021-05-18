pub fn hamming_distance<T>(a: &T, b: &T) -> isize
where
    T: num::PrimInt,
{
    (*a ^ *b).count_ones() as isize
}

pub fn levenshtein_distance<S: AsRef<str>>(a: &S, b: &S) -> isize {
    let a = a.as_ref();
    let b = b.as_ref();

    if a == b {
        return 0;
    }

    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len as isize;
    }

    if b_len == 0 {
        return a_len as isize;
    }

    let mut res = 0;
    let mut cache: Vec<usize> = (1..).take(a_len).collect();
    let mut a_dist;
    let mut b_dist;

    for (ib, cb) in b.chars().enumerate() {
        res = ib;
        a_dist = ib;
        for (ia, ca) in a.chars().enumerate() {
            b_dist = if ca == cb { a_dist } else { a_dist + 1 };
            a_dist = cache[ia];

            res = if a_dist > res {
                if b_dist > res {
                    res + 1
                } else {
                    b_dist
                }
            } else if b_dist > a_dist {
                a_dist + 1
            } else {
                b_dist
            };

            cache[ia] = res;
        }
    }

    res as isize
}

#[derive(Debug, Eq, PartialEq)]
struct Node<T> {
    word: T,
    children: Vec<(isize, Node<T>)>,
}

pub struct BkTree<T> {
    root: Option<Box<Node<T>>>,
    dist: Box<dyn Fn(&T, &T) -> isize>,
}

impl<T> BkTree<T> {
    pub fn new(dist: impl Fn(&T, &T) -> isize + 'static) -> Self {
        Self {
            root: None,
            dist: Box::new(dist),
        }
    }

    pub fn insert_all<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for i in iter {
            self.insert(i);
        }
    }

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

    pub fn into_iter(self) -> IntoIter<T> {
        let mut queue = Vec::new();
        if let Some(root) = self.root {
            queue.push(*root);
        }
        IntoIter { queue }
    }
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
