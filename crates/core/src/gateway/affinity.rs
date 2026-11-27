use std::{collections::VecDeque, sync::Mutex, time::Duration};
use tokio::time::Instant;

const MAX_TURNS: usize = 256;
const MAX_AGE: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Key {
    pub group: usize,
    pub turn: [u8; 32],
}

#[derive(Clone)]
pub(super) struct Decision {
    pub rule: Option<usize>,
    pub intent: Option<String>,
    pub effort: Option<&'static str>,
}

struct Turn {
    key: Key,
    decision: Decision,
    started: Instant,
}

#[derive(Default)]
pub(super) struct Cache(Mutex<VecDeque<Turn>>);

impl Cache {
    pub fn get(&self, key: Key) -> Option<Decision> {
        let turns = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let now = Instant::now();
        turns
            .iter()
            .find(|turn| turn.key == key && now - turn.started < MAX_AGE)
            .map(|turn| turn.decision.clone())
    }

    pub fn remember(&self, key: Key, decision: Decision, replaced: Option<Option<usize>>) {
        let mut turns = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let now = Instant::now();
        while turns
            .front()
            .is_some_and(|turn| now - turn.started >= MAX_AGE)
        {
            turns.pop_front();
        }
        if let Some(turn) = turns.iter_mut().find(|turn| turn.key == key) {
            if replaced == Some(turn.decision.rule) {
                turn.decision = decision;
            }
            return;
        }
        if turns.len() == MAX_TURNS {
            turns.pop_front();
        }
        turns.push_back(Turn {
            key,
            decision,
            started: now,
        });
    }
}
