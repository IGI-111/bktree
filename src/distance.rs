pub trait Distance<T: ?Sized> {
    fn distance(&self, a: &T, b: &T) -> isize;
}

#[cfg_attr(
    feature = "serde-support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug)]
pub struct HammingDistance;

#[cfg_attr(
    feature = "serde-support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug)]
pub struct LevenshteinDistance;

impl<T: AsRef<str> + ?Sized> Distance<T> for LevenshteinDistance {
    fn distance(&self, a: &T, b: &T) -> isize {
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
}

impl<T: num::PrimInt + ?Sized> Distance<T> for HammingDistance {
    fn distance(&self, a: &T, b: &T) -> isize {
        (*a ^ *b).count_ones() as isize
    }
}
