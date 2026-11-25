use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

pub(super) enum Round {
    Rotate(AtomicUsize),
    Weighted {
        weights: Vec<u32>,
        scores: Mutex<Vec<i64>>,
    },
}

impl Round {
    pub(super) fn weighted(weights: Vec<u32>) -> Self {
        Self::Weighted {
            scores: Mutex::new(vec![0; weights.len()]),
            weights,
        }
    }

    pub(super) fn order<T>(&self, eligible: &mut [(usize, T)]) {
        if eligible.is_empty() {
            return;
        }
        match self {
            Self::Rotate(turn) => {
                let first = turn.fetch_add(1, Ordering::Relaxed) % eligible.len();
                eligible.rotate_left(first);
            }
            Self::Weighted { weights, scores } => {
                let mut scores = scores.lock().expect("group routing lock");
                let mut best = 0;
                let mut total = 0;
                for (position, (index, _)) in eligible.iter().enumerate() {
                    let weight = i64::from(weights[*index]);
                    scores[*index] += weight;
                    total += weight;
                    if scores[*index] > scores[eligible[best].0] {
                        best = position;
                    }
                }
                scores[eligible[best].0] -= total;
                // Move the winner to the front and preserve listed fallback order.
                eligible[..=best].rotate_right(1);
            }
        }
    }
}
