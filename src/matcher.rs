use std::cell::RefCell;
use std::mem::swap;

pub trait Matcher {
    // Returns a normalized score [0.0, 1.0] where 1.0 is
    // the best possible match and 0.0 is the lowest
    fn score(&self, query: &str, candidate: &str) -> f32;
}

// Levenshtein struct is not thread safe.
pub struct Levenshtein {
    prev: RefCell<Vec<usize>>,
    curr: RefCell<Vec<usize>>,
}

impl Levenshtein {
    pub fn new() -> Self {
        Self {
            prev: RefCell::new(Vec::new()),
            curr: RefCell::new(Vec::new()),
        }
    }
}

impl Matcher for Levenshtein {
    fn score(&self, query: &str, candidate: &str) -> f32 {
        let q = query.as_bytes();
        let c = candidate.as_bytes();

        let n = q.len();
        let m = c.len();

        if n == 0 && m == 0 {
            return 1.0;
        }
        if m == 0 || m == 0 {
            return 0.0;
        }

        let mut prev = self.prev.borrow_mut();
        let mut curr = self.curr.borrow_mut();

        if prev.len() < m + 1 {
            prev.resize(m + 1, 0);
        }
        if curr.len() < m + 1 {
            curr.resize(m + 1, 0);
        }

        for j in 0..=m {
            prev[j] = j;
        }

        for i in 1..=n {
            curr[0] = i;
            let qi = q[i - 1];

            for j in 1..=m {
                let cost = if qi == c[j - 1] { 0 } else { 1 };

                let deletion = prev[j] + 1;
                let insertion = curr[j - 1] + 1;
                let substitution = prev[j - 1] + cost;

                curr[j] = deletion.min(insertion).min(substitution);
            }

            swap(&mut *prev, &mut *curr);
        }

        let distance = prev[m] as f32;
        let max_len = n.max(m) as f32;

        1.0 - (distance / max_len)
    }
}
