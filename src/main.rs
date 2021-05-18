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

    pub fn find(&self, val: T, max_dist: isize) -> Vec<&T> {
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
                        found.push(&n.word);
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
}

fn main() {
    let mut bk = BkTree::new(|w1: &String, w2: &String| {
        let (big, sml) = if w1.len() < w2.len() {
            (w2, w1)
        } else {
            (w1, w2)
        };
        let mut dist = big.len() as isize - sml.len() as isize;
        for (bc, sc) in big.chars().zip(sml.chars()) {
            if bc != sc {
                dist += 1;
            }
        }
        dist
    });
    bk.insert("book".to_string());
    bk.insert("books".to_string());
    bk.insert("boo".to_string());
    bk.insert("boon".to_string());
    bk.insert("cook".to_string());
    bk.insert("cake".to_string());
    bk.insert("cape".to_string());
    bk.insert("cart".to_string());
    println!("{:?}", bk.find("bo".into(), 2));
}

pub fn levenshtein_distance<T>(a: &T, b: &T) -> isize {
    for c in a.chars() {}
}
