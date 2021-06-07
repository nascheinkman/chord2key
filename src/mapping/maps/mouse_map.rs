use crate::constants::*;
use crate::events::*;
use crate::input::events::*;
use crate::mapping::actions::*;
use crate::mapping::thresholds::*;
use crate::output::actions::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The input type used for Mouse mapping
///
/// ThresholdedAxis is used in order to implement deadzones
pub type MouseInput = ThresholdedAxis;

/// A list of tuples of mapping [MouseInput] to [MouseProfile]
pub type MouseMapInput = Vec<(MouseInput, MouseProfile)>;

/// A linear acceleration profile for Mouse movement
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct MouseProfile {
    /// The [RelAxisCode] to be used when emitting mouse events
    pub code: RelAxisCode,

    /// The linear sensitivity of the mouse.
    ///
    /// The mouse velocity follows the following formula:
    /// ```ignore
    /// mouse_velocity = MouseProfile.slope * (axis_value - threshold_value) + MouseProfile.offset
    /// ```
    pub slope: f64, // m in y = mx + b

    /// The linear offset of the mouse.
    /// The mouse velocity follows the following formula:
    /// ```ignore
    /// mouse_velocity = MouseProfile.slope * (axis_value - threshold_value) + MouseProfile.offset
    /// ```
    pub offset: f64, // b in y = mx + b
}

pub struct MouseMap {
    mouse_mapping: HashMap<MouseInput, MouseProfile>,
    axis_thresholds: AllAxisThresholds,
    axis_states: HashMap<AbsAxisCode, Option<ThresholdType>>,
}

impl MouseProfile {
    pub fn map_state_to_action(&self, state: AxisState) -> Action {
        let mouse_state: AxisState = (self.slope * (state as f64) + self.offset) as AxisState;
        StateChange::new(None, Some(vec![(self.code, mouse_state)].into())).into()
    }
    pub fn zeroed(&self) -> Action {
        StateChange::new(None, Some(vec![(self.code, 0)].into())).into()
    }
}

impl MouseMap {
    fn handle_axis(&mut self, ev: &AbsAxisEvent) -> Option<(Action, Option<Action>)> {
        let prev_state = self.axis_states.get(&ev.axis()).copied().unwrap_or(None);
        match self.axis_thresholds.get_passing_with_state(ev) {
            Some((t_axis, t_val)) => match prev_state {
                Some(prev_threshold) => {
                    if prev_threshold == t_axis.threshold() {
                        // Axis hasn't changed threshold
                        let axis_val = ev.state() - t_val;
                        self.mouse_mapping
                            .get(&t_axis)
                            .map(|profile| (profile.map_state_to_action(axis_val), None))
                    } else {
                        // Axis swapped threshold
                        let axis_val = ev.state() - t_val;
                        let prev_act = self
                            .mouse_mapping
                            .get(&(ev.axis(), prev_threshold).into())
                            .copied()
                            .map(|p| p.zeroed());
                        let new_act = self
                            .mouse_mapping
                            .get(&t_axis)
                            .map(|profile| profile.map_state_to_action(axis_val));

                        self.axis_states.insert(ev.axis(), Some(t_axis.threshold()));
                        match new_act {
                            Some(new_act) => Some((new_act, prev_act)),
                            None => prev_act.map(|act| (act, None)),
                        }
                    }
                }
                None => {
                    // Axis passed new threshold
                    let axis_val = ev.state() - t_val;
                    self.axis_states.insert(ev.axis(), Some(t_axis.threshold()));
                    self.mouse_mapping
                        .get(&t_axis)
                        .map(|profile| (profile.map_state_to_action(axis_val), None))
                }
            },
            None => match prev_state {
                Some(prev_threshold) => {
                    // Axis receded from threshold
                    self.axis_states.insert(ev.axis(), None);
                    self.mouse_mapping
                        .get(&(ev.axis(), prev_threshold).into())
                        .copied()
                        .map(|p| (p.zeroed(), None))
                }
                None => None,
            },
        }
        /*
        let prev_state = self.axis_states.get(&ev.axis()).copied().unwrap_or(None);
        match self.axis_thresholds.get_passing(ev) {
            Some(thresholded_axis) => match prev_state {
                Some(prev_threshold) => {
                    if prev_threshold == thresholded_axis.threshold() {
                        // Axis hasn't changed threshold
                        None
                    } else {
                        // Axis swapped threshold
                        let prev_act = self
                            .modifier_mapping
                            .get(&ModifierInput::Axis((ev.axis(), prev_threshold).into()))
                            .cloned();
                        let new_act = self.modifier_mapping.get(&thresholded_axis.into()).cloned();
                        self.axis_states
                            .insert(ev.axis(), Some(thresholded_axis.threshold()));
                        match new_act {
                            Some(new_act) => Some((new_act, prev_act)),
                            None => prev_act.map(|act| (act, None)),
                        }
                    }
                }
                None => {
                    // Axis passed new threshold
                    self.axis_states
                        .insert(ev.axis(), Some(thresholded_axis.threshold()));
                    self.modifier_mapping
                        .get(&thresholded_axis.into())
                        .cloned()
                        .map(|act| (act, None))
                }
            },
            None => match prev_state {
                Some(threshold_type) => {
                    // Axis receded from threshold
                    self.axis_states.insert(ev.axis(), None);
                    self.modifier_mapping
                        .get(&ModifierInput::Axis((ev.axis(), threshold_type).into()))
                        .cloned()
                        .map(|act| (act, None))
                }
                None => None,
            },
        }
        */
    }
    pub fn handle_event(&mut self, ev: &InputEvent) -> Option<(Action, Option<Action>)> {
        match ev {
            InputEvent::KeyEvent(_) => None,
            InputEvent::AbsAxisEvent(aev) => self.handle_axis(aev),
            InputEvent::RelAxisEvent(_) => None,
        }
    }

    pub fn init(mouse_map: MouseMapInput, all_a_t: AllAxisThresholds) -> Self {
        let mut mouse_mapping = HashMap::<MouseInput, MouseProfile>::with_capacity(mouse_map.len());
        let mut axis_states = HashMap::<AbsAxisCode, Option<ThresholdType>>::new();
        for (input, output) in mouse_map {
            mouse_mapping.insert(input, output);
            axis_states.insert(input.code(), None);
        }
        Self {
            mouse_mapping,
            axis_thresholds: all_a_t,
            axis_states,
        }
    }
}
