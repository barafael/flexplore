use bevy::prelude::*;

use flexplore::config::FlexConfig;

const MAX_UNDO: usize = 100;

#[derive(Resource)]
pub struct UndoHistory {
    snapshots: Vec<FlexConfig>,
    cursor: usize,
}

impl UndoHistory {
    pub fn new(initial: FlexConfig) -> Self {
        Self {
            snapshots: vec![initial],
            cursor: 0,
        }
    }

    /// Record the current state after a mutation.
    pub fn push(&mut self, state: FlexConfig) {
        self.snapshots.truncate(self.cursor + 1);
        self.snapshots.push(state);
        self.cursor += 1;
        if self.snapshots.len() > MAX_UNDO {
            let excess = self.snapshots.len() - MAX_UNDO;
            self.snapshots.drain(0..excess);
            self.cursor -= excess;
        }
    }

    pub fn undo(&mut self) -> Option<&FlexConfig> {
        if self.cursor > 0 {
            self.cursor -= 1;
            Some(&self.snapshots[self.cursor])
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&FlexConfig> {
        if self.cursor + 1 < self.snapshots.len() {
            self.cursor += 1;
            Some(&self.snapshots[self.cursor])
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.snapshots.len()
    }
}
