//! Minecraft's saved function timer queue. Time ties keep insertion order.
use super::{DatapackGraph, ScheduledFunction, function_id};
use law::Refusal;
use std::collections::VecDeque;

pub struct FunctionClock {
    now: i64,
    queue: VecDeque<ScheduledFunction>,
}

impl FunctionClock {
    pub fn saved(graph: &DatapackGraph) -> Self {
        Self { now: graph.game_tick, queue: graph.scheduled.iter().cloned().collect() }
    }

    pub fn advance_to(&mut self, tick: i64) -> Result<(), Refusal> {
        if tick < self.now {
            return Err(Refusal(format!("datapack game clock moved backwards from {} to {tick}", self.now)));
        }
        self.now = tick;
        Ok(())
    }

    pub fn next_due(&mut self) -> Option<ScheduledFunction> {
        self.queue.pop_front_if(|event| event.trigger_tick <= self.now)
    }

    /// ScheduleCommand clears the target on replace and de-duplicates
    /// (event ID, trigger time) on append. Command results are unavailable.
    pub fn command(&mut self, graph: &DatapackGraph, args: &[&str]) -> Result<(), Refusal> {
        let refused = || Refusal(format!("unsupported function schedule at tick {}", self.now));
        let (target, delay, replace) = match args {
            ["clear", id] => {
                self.queue.retain(|event| event.id != *id);
                return Ok(());
            }
            ["function", target, delay] | ["function", target, delay, "replace"] => (*target, *delay, true),
            ["function", target, delay, "append"] => (*target, *delay, false),
            _ => return Err(refused()),
        };
        let target = function_id(target);
        if let Some(tag) = target.strip_prefix('#') {
            if !graph.tags.contains_key(tag) { return Err(refused()); }
        } else if !graph.functions.get(&target).is_some_and(|lines| {
            !lines.iter().any(|line| line.verb().starts_with('$'))
        }) {
            return Err(refused());
        }
        let delay = time_argument(delay).ok_or_else(refused)?;
        let trigger_tick = self.now.checked_add(i64::from(delay)).ok_or_else(refused)?;
        let id = target.clone();
        if replace {
            self.queue.retain(|event| event.id != id);
        } else if self.queue.iter().any(|event| event.id == id && event.trigger_tick == trigger_tick) {
            return Ok(());
        }
        let index = self.queue.partition_point(|event| event.trigger_tick <= trigger_tick);
        self.queue.insert(index, ScheduledFunction { trigger_tick, id, target });
        Ok(())
    }
}

fn time_argument(word: &str) -> Option<i32> {
    let (value, scale) = word.strip_suffix('t').map(|n| (n, 1))
        .or_else(|| word.strip_suffix('s').map(|n| (n, 20)))
        .or_else(|| word.strip_suffix('d').map(|n| (n, 24_000)))
        .unwrap_or((word, 1));
    let ticks = value.parse::<f32>().ok()? * scale as f32;
    if !ticks.is_finite() { return None; }
    let ticks = law::math::round(ticks);
    (ticks > 0).then_some(ticks)
}
