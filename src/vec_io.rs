use std::collections::VecDeque;

use crate::interpreter::{Input, LNCInput, Output};

#[derive(Default)]
pub struct QueueInput {
    pub queue: VecDeque<LNCInput>,
}

impl QueueInput {
    pub fn new(nums: &[usize]) -> Result<Self, String> {
        let mut queue = VecDeque::new();

        for num in nums {
            if let Some(lnc_num) = LNCInput::new(*num) {
                queue.push_back(lnc_num);
            } else {
                return Err(format!("error: input number ({num}) is too large"));
            }
        }

        Ok(Self { queue })
    }
}

impl Input for QueueInput {
    fn take(&mut self) -> Result<LNCInput, String> {
        if let Some(lnc_num) = self.queue.pop_front() {
            Ok(lnc_num)
        } else {
            Err("error: input queue is empty!".into())
        }
    }
}

#[derive(Default)]
pub struct StackOutput {
    pub stack: Vec<usize>,
}

impl Output for StackOutput {
    fn send(&mut self, val: usize) {
        self.stack.push(val);
    }
}
