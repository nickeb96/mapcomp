
//! Macros for container comprehensions similar to Python's list comprehension.
//!
//! This crate adds vector, set, map, and generator comprehensions.  It is
//! meant to complement [maplit](https://docs.rs/maplit/) which provides
//! macro literals for the same standard containers.
//!
//! ```rust
//! # #![feature(match_default_bindings)]
//! # #[macro_use] extern crate mapcomp;
//! # fn main() {
//! let v = vec![3, 2, 6, 9, 5];
//!
//! let even_squares = vecc![x * x; for x in v.iter(); if x % 2 == 0];
//!
//! assert_eq!(even_squares, vec![4, 36]);
//! # }
//! ```
//!
//! The macro names are the same as maplit's container literal macros but with
//! a **c** at the end for **c**omprehension.  There is an additional macro
//! `iterc!()` for creating lazily evaluated generator expressions.
//!
//! List comprehensions exist [in many languages](https://en.wikipedia.org/wiki/List_comprehension)
//! and in many styles.  This crate uses the same syntax as Python's list
//! comprehensions due to it's ease of use, readability, and established
//! popularity.  If you understand Python's list comprehension, then you
//! understand `mapcomp`'s comprehensions.
//!
//! One minor deviation from Python's syntax is the presence of semicolons
//! between clauses which is a limitation of Rust's macro language.  Another
//! difference is that the map comprehensions use the `=>` token to separate
//! the key from the value instead of a colon the way Python does it.
//!
//! The `for var in iteratable` clause can be nested as many times as you want
//! and the `if conditional` is optional after each loop clause.
//!
//! ```rust
//! # #[macro_use] extern crate mapcomp;
//! # fn main() {
//! let grid = vec![vec![-2, 5], vec![3, -7, 6], vec![4, 2]];
//!
//! let v = vecc![x; for row in grid; if row.len() < 3; for x in row; if x > 0];
//!
//! assert_eq!(v, vec![5, 4, 2]);
//! # }
//! ```


#![feature(generators, generator_trait, arbitrary_self_types)]


/// This is an implementation detail used by `iterc!()` and it should not be
/// directly instantiated.
#[doc(hidden)]
pub struct GeneratorIterator<G: ::std::ops::Generator + ::std::marker::Unpin> {
    generator: G,
}

impl<G: ::std::ops::Generator + ::std::marker::Unpin> GeneratorIterator<G> {
    pub fn new(generator: G) -> GeneratorIterator<G> {
        GeneratorIterator { generator }
    }
}

impl<G: ::std::ops::Generator + ::std::marker::Unpin> Iterator for GeneratorIterator<G> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        use ::std::ops::GeneratorState;
        match ::std::pin::Pin::new(&mut self.generator).resume() {
            GeneratorState::Yielded(y) => Some(y),
            _ => None,
        }
    }
}



/// Iterator Comprehension
///
/// Returns an iterator over the contents of the comprehension.  It is
/// analogous to [Python's generator comprehensions](https://www.python.org/dev/peps/pep-0289/).
/// Syntactically, it is similar to the `vecc![]` macro but it returns a lazily
/// evaluated iterator instead of a container.  It's use requires the experimental
/// generators feature.
///
/// ```rust
/// #![feature(generators, generator_trait)]
/// #[macro_use]
/// extern crate mapcomp;
///
/// # fn main() {
/// let numbers = [8, 3, 5, 7];
///
/// let mut powers_of_two = iterc!(1 << x; for x in &numbers);
///
/// assert_eq!(Some(256), powers_of_two.next());
/// assert_eq!(Some(8), powers_of_two.next());
/// assert_eq!(Some(32), powers_of_two.next());
/// assert_eq!(Some(128), powers_of_two.next());
/// assert_eq!(None, powers_of_two.next());
/// # }
/// ```
///
/// Since it only creates an iterator and not a fully populated container, the
/// comprehension can be created over an unbounded or infinite iterator.
///
/// ```rust
/// # #![feature(generators, generator_trait)]
/// # #[macro_use] extern crate mapcomp;
/// # fn main() {
/// let mut odd_squares = iterc!(x * x; for x in 1..; if x % 2 == 1);
///
/// assert_eq!(Some(1), odd_squares.next());
/// assert_eq!(Some(9), odd_squares.next());
/// assert_eq!(Some(25), odd_squares.next());
/// assert_eq!(Some(49), odd_squares.next());
/// # }
/// ```
#[macro_export]
macro_rules! iterc {
    (@__ $exp:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                yield $exp;
            }
        }
    );

    (@__ $exp:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            yield $exp;
        }
    );

    (@__ $exp:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                iterc!(@__ $exp; $($tail)+)
            }
        }
    );

    (@__ $exp:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            iterc!(@__ $exp; $($tail)+)
        }
    );

    ($exp:expr; $($tail:tt)+) => ({
        let mut generator = || {
            iterc!(@__ $exp; $($tail)+)
        };
        ::mapcomp::GeneratorIterator::new(generator)
    });
}



/// Vector Comprehension
///
/// Creates a new `Vec` from the contents of the comprehension.  Vector
/// comprehensions are analogous to Python's list comprehension.
///
/// Python code:
///
/// ```python
/// items = [4, 7, 2]
///
/// even_squares = [x*x for x in items if x % 2 == 0]
/// ```
///
/// Rust equivalent code:
///
/// ```
/// # #[macro_use] extern crate mapcomp; fn main() {
/// let items = [4, 7, 2];
///
/// let even_squares = vecc![x*x; for x in &items; if x % 2 == 0];
/// 
/// assert_eq!(even_squares, vec![16, 4]);
/// # }
/// ```
#[macro_export]
macro_rules! vecc {
    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                $acc.push($exp);
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            $acc.push($exp);
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                vecc![@__ $acc, $exp; $($tail)+];
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            vecc![@__ $acc, $exp; $($tail)+];
        }
    );

    ($exp:expr; $($tail:tt)+) => ({
        let mut ret = ::std::vec::Vec::new();
        vecc![@__ ret, $exp; $($tail)+];
        ret
    });
}



/// Hash Set Comprehension
///
/// Creates a `HashSet` from the contents of the comprehension.  It is
/// analogous to Python's set comprehension.
///
/// Python code:
///
/// ```python
/// matrix = [[3, 8, 7], [9, 5, 3], [4, 5, 6]]
///
/// members = {n for row in matrix for n in row}
/// ```
///
/// Rust equivalent code:
///
/// ```
/// # #[macro_use] extern crate mapcomp;
/// # fn main() {
/// let matrix = [[3, 8, 7], [9, 5, 3], [4, 5, 6]];
///
/// let members = hashsetc!{n; for row in &matrix; for n in row};
///
/// for n in &[3, 8, 7, 9, 5, 4, 6] {
///     assert!(members.contains(n));
/// }
/// # }
/// ```
#[macro_export]
macro_rules! hashsetc {
    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                $acc.insert($exp);
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            $acc.insert($exp);
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                hashsetc!{@__ $acc, $exp; $($tail)+};
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            hashsetc!{@__ $acc, $exp; $($tail)+};
        }
    );

    ($exp:expr; $($tail:tt)+) => ({
        let mut ret = ::std::collections::HashSet::new();
        hashsetc!{@__ ret, $exp; $($tail)+};
        ret
    });
}



/// Hash Map Comprehension
///
/// Creates a `HashMap` from the contents of the comprehension.  It is
/// analogous to Python's dictionary comprehension except that it uses the `=>`
/// token instead of a colon.
///
/// Python code:
///
/// ```python
/// numbers = [6, 4, 18]
///
/// halves = {str(x): x / 2 for x in numbers}
/// ```
///
/// Rust equivalent code:
///
/// ```rust
/// # #[macro_use] extern crate mapcomp;
/// # fn main() {
/// let numbers = [6, 4, 18];
///
/// let halves = hashmapc!{x.to_string() => x / 2; for x in numbers.iter()};
///
/// for &(k, v) in &[("6", 3), ("4", 2), ("18", 9)] {
///     assert_eq!(halves[k], v);
/// }
/// # }
/// ```
#[macro_export]
macro_rules! hashmapc {
    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                $acc.insert($key, $val);
            }
        }
    );

    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            $acc.insert($key, $val);
        }
    );

    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                hashmapc!{@__ $acc, $key => $val; $($tail)+};
            }
        }
    );

    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            hashmapc!{@__ $acc, $key => $val; $($tail)+};
        }
    );

    ($key:expr => $val:expr; $($tail:tt)+) => ({
        let mut ret = ::std::collections::HashMap::new();
        hashmapc!{@__ ret, $key => $val; $($tail)+};
        ret
    });
}



/// BTree Set Comprehension
///
/// Creates a `BTreeSet` from the contents of the comprehension.  Same syntax as
/// `hashsetc!{}`.
///
/// ```rust
/// # #![feature(match_default_bindings)]
/// # #[macro_use] extern crate mapcomp;
/// # fn main() {
/// let pairs = btreesetc!{(i, j); for i in 4..7; for j in 10..12};
///
/// for i in 4..7 {
///     for j in 10..12 {
///         assert!(pairs.contains(&(i, j)));
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! btreesetc {
    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                $acc.insert($exp);
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            $acc.insert($exp);
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                btreesetc!{@__ $acc, $exp; $($tail)+};
            }
        }
    );

    (@__ $acc:ident, $exp:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            btreesetc!{@__ $acc, $exp; $($tail)+};
        }
    );

    ($exp:expr; $($tail:tt)+) => ({
        let mut ret = ::std::collections::BTreeSet::new();
        btreesetc!{@__ ret, $exp; $($tail)+};
        ret
    });
}



/// BTree Map Comprehension
///
/// Creates a `BTreeMap` from the contents of the comprehension.  Same syntax
/// as `hashmapc!{}`.
///
/// ```rust
/// # #[macro_use] extern crate mapcomp;
/// # fn main() {
/// let array = [5, 3, 9, 6];
///
/// let index_map = hashmapc!{x => i; for (i, x) in array.iter().enumerate()};
///
/// for (i, x) in array.iter().enumerate() {
///     assert_eq!(index_map[x], i);
/// }
/// # }
/// ```
#[macro_export]
macro_rules! btreemapc {
    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr; if $cond:expr) => (
        for $item in $iter {
            if $cond {
                $acc.insert($key, $val);
            }
        }
    );

    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr) => (
        for $item in $iter {
            $acc.insert($key, $val);
        }
    );

    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr; if $cond:expr; $($tail:tt)+) => (
        for $item in $iter {
            if $cond {
                btreemapc!{@__ $acc, $key => $val; $($tail)+};
            }
        }
    );

    (@__ $acc:ident, $key:expr => $val:expr; for $item:pat in $iter:expr; $($tail:tt)+) => (
        for $item in $iter {
            btreemapc!{@__ $acc, $key => $val; $($tail)+};
        }
    );

    ($key:expr => $val:expr; $($tail:tt)+) => ({
        let mut ret = ::std::collections::BTreeMap::new();
        btreemapc!{@__ ret, $key => $val; $($tail)+};
        ret
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let _v = vecc![x * x; for x in 1..10; if x % 2 == 0];
    }
}

