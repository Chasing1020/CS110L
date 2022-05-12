/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

/*
Implement add_n, which takes a vector of numbers and some number n.
The function should return a new vector whose elements are the numbers
in the original vector v with n added to each number.
*/
fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    let mut newv = vec![];
    for i in v.iter() {
        newv.push(i + n);
    }
    newv
}

/*
Implement add_n_inplace, which does the same thing as add_n,
but modifies v directly (in place) and does not return anything.
*/
fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    v.iter_mut().for_each(|i| *i += n);
}

fn add_n_inplace2(v: &mut Vec<i32>, n: i32) {
    for i in 0..v.len() {
        v[i] = v[i] + n;
    }
}

/*
Implement dedup that removes duplicate elements from a vector in-place (i.e. modifies v directly).
If an element is repeated anywhere in the vector, you should keep the element 
that appears first. You may want to use a HashSet for this.
*/
fn dedup(v: &mut Vec<i32>) {
    let set: HashSet<_> = v.drain(..).collect(); // dedup
    v.extend(set.into_iter());
}

// Note Vec#dedup only removes consecutive elements from a vector
fn dedup2(v: &mut Vec<i32>) {
    let mut set = HashSet::new();
    v.retain(|e| set.insert(*e));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1, 2, 3];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3, 4, 5]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
