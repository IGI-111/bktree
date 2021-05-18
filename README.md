# bktree

[![Crates.io](https://img.shields.io/crates/v/bktree.svg)](https://crates.io/crates/bktree)

A crate implementing a Brukhard Keller tree datastructure which allows for fast querying of
"close" matches on discrete distances.

Useful for spell checking based on edit distance and other typical applications.

```rust
use bktree::*;

let mut bk = BkTree::new(hamming_distance);
bk.insert_all(vec![0, 4, 5, 14, 15]);

let (words, dists): (Vec<i32>, Vec<isize>) = bk.find(13, 1).into_iter().unzip();
assert_eq!(words, [5, 15]);
assert_eq!(dists, [1, 1]);
```

```rust
use bktree::*;

let mut bk = BkTree::new(levenshtein_distance);
bk.insert_all(vec![
    "book", "books", "boo", "boon", "cook", "cake", "cape", "cart",
]);
let (words, dists): (Vec<&str>, Vec<isize>) = bk.find("bo", 2).into_iter().unzip();
assert_eq!(words, ["book", "boo", "boon"]);
assert_eq!(dists, [2, 1, 2]);
```
