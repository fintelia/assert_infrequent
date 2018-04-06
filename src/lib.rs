#![feature(test)]

#[macro_use]
extern crate lazy_static;

extern crate backtrace;
extern crate test;

use backtrace::Backtrace;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Hash, PartialEq, Eq)]
struct Trace(Vec<usize>);

lazy_static! {
    static ref BACKTRACES: Mutex<HashMap<Trace, usize>> = Mutex::new(HashMap::new());
}

/// Asserts that the given function is called at most `n` times from any given point.
///
/// # Example 1:
/// ```
/// fn main() {
///     assert_infrequent::at_most(1); // Ok, first call to at_most
///     assert_infrequent::at_most(1); // Ok, first call to at_most at this callsite
///     assert_infrequent::at_most(3); // Ok, 1 < 3.
///
///     for _ in 0..5 {
///         assert_infrequent::at_most(5); // Ok, only hit 5 times.
///     }
/// }
/// ```
///
/// # Example 2:
/// ```rust,should_panic
/// fn expensive_computation() -> u64 {
///     assert_infrequent::at_most(1);
///     // ...
///     # 0
/// }
///
/// fn main() {
///     for _ in 0..2 {
///         let v = expensive_computation(); // Panic!
///         // ...
///     }
/// }
pub fn at_most(n: usize) {
    let hits = {
        let mut backtraces = BACKTRACES.lock().unwrap();
        let backtrace = Trace(
            Backtrace::new_unresolved()
                .frames()
                .iter()
                .map(|f| f.ip() as usize)
                .collect(),
        );
        let hits = backtraces.entry(backtrace).or_insert(0);
        *hits += 1;
        *hits
    };
    assert!(hits <= n);
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    #[test]
    fn it_works() {
        super::at_most(1);
        super::at_most(1);
        super::at_most(1);
        super::at_most(3);

        for _ in 0..5 {
            super::at_most(5);
        }
    }

    #[test]
    #[should_panic]
    fn never_call() {
        super::at_most(0);
    }

    #[test]
    #[should_panic]
    fn in_loop() {
        for _ in 0..5 {
            super::at_most(4);
        }
    }

    #[bench]
    fn bench(b: &mut Bencher) {
        b.iter(|| super::at_most(1000_000_000));
    }
}
