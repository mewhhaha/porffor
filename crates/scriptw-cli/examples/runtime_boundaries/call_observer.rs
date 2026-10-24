//! Bounded diagnostic observer of repeated guest calls in one live Realm.
//! It never invokes a JS interpreter, resets the Realm or subtracts a control.
use scriptw_engine::HostHooks;
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

const PREFIX: &str = "scriptw:call:";
pub(super) const READY: &str = "scriptw:call:ready";

#[derive(Default)]
struct State {
    times: Vec<(Instant, Instant)>,
    error: Option<String>,
    finished: bool,
}

#[derive(Clone)]
pub(super) struct Observer {
    samples: usize,
    state: Arc<Mutex<State>>,
}

impl Observer {
    pub(super) fn new(samples: usize) -> Result<Self, String> {
        if !(7..=100).contains(&samples) {
            return Err("call samples must be 7..100".into());
        }
        Ok(Self {
            samples,
            state: Arc::new(Mutex::new(State::default())),
        })
    }

    fn marker(&self, position: usize) -> Option<String> {
        if position == 0 {
            return Some(READY.into());
        }
        let index = (position - 1) / 2;
        (index < self.samples).then(|| {
            format!(
                "{PREFIX}{}:{index}",
                if position % 2 == 1 { "start" } else { "end" }
            )
        })
    }

    pub(super) fn finish(&self) -> Result<Value, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "call observer mutex poisoned")?;
        if let Some(error) = &state.error {
            return Err(error.clone());
        }
        if state.finished || state.times.len() != 1 + 2 * self.samples {
            return Err("call observer requires one ready and every ordered start/end pair".into());
        }
        state.finished = true;
        let origin = state.times[0].0;
        let spans = (0..self.samples)
            .map(|index| {
                let start = state.times[1 + 2 * index];
                let end = state.times[2 + 2 * index];
                end.0.duration_since(start.1).as_nanos()
            })
            .collect::<Vec<_>>();
        Ok(json!({
            "samples_ns": spans,
            "callbacks": state.times.iter().enumerate().map(|(position, (entered, exited))|
                json!({"marker": self.marker(position), "entry_ns": entered.duration_since(origin).as_nanos(),
                    "exit_ns": exited.duration_since(origin).as_nanos()})).collect::<Vec<_>>(),
            "boundary": "Recorded end of start-marker host callback to entry of end-marker callback. Includes guest marker return, one compiled JS function call and end-marker argument/print entry; excludes setup, ten guest warmups and checksum printing. No subtraction. Wall time, not isolated CPU/allocation time."
        }))
    }
}

impl HostHooks for Observer {
    fn print_line(&self, text: &str) {
        if !text.starts_with(PREFIX) {
            return;
        }
        let entered = Instant::now();
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.error.is_some() {
            return;
        }
        let position = state.times.len();
        if state.finished || self.marker(position).as_deref() != Some(text) {
            state.error = Some(format!(
                "unexpected call marker at position {position}: {text}"
            ));
            return;
        }
        state.times.push((entered, Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_only_complete_ordered_pairs_and_keeps_callback_observations() {
        let observer = Observer::new(7).unwrap();
        observer.print_line("warmup:9:42");
        observer.print_line(READY);
        for index in 0..7 {
            observer.print_line(&format!("scriptw:call:start:{index}"));
            observer.print_line(&format!("scriptw:call:end:{index}"));
            observer.print_line(&format!("sample:{index}:42"));
        }
        let result = observer.finish().unwrap();
        let callbacks = result["callbacks"].as_array().unwrap();
        for index in 0..7 {
            assert_eq!(
                result["samples_ns"][index].as_u64().unwrap(),
                callbacks[2 + 2 * index]["entry_ns"].as_u64().unwrap()
                    - callbacks[1 + 2 * index]["exit_ns"].as_u64().unwrap()
            );
        }
        assert!(observer.finish().is_err());
    }

    #[test]
    fn missing_repeated_out_of_order_and_excess_markers_reject() {
        for markers in [
            vec![],
            vec![READY],
            vec![READY, READY],
            vec!["scriptw:call:start:0"],
            vec![READY, "scriptw:call:end:0"],
            vec![READY, "scriptw:call:start:1"],
            vec![READY, "scriptw:call:other"],
        ] {
            let observer = Observer::new(7).unwrap();
            for marker in markers {
                observer.print_line(marker);
            }
            assert!(observer.finish().is_err());
        }
        assert!(Observer::new(6).is_err());
        assert!(Observer::new(101).is_err());
        let excess = Observer::new(7).unwrap();
        for position in 0..15 {
            excess.print_line(&excess.marker(position).unwrap());
        }
        excess.print_line("scriptw:call:start:7");
        assert!(excess.finish().is_err());
    }
}
