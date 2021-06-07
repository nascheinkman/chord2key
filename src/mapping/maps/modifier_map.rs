use crate::constants::*;
use crate::events::*;
use crate::input::events::InputEvent;
use crate::mapping::actions::*;
use crate::mapping::thresholds::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A list of tuples that map [ModifierInput] to an [Action].
pub type ModifierMapInput = Vec<(ModifierInput, Action)>;

/// The input types used for modifiers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ModifierInput {
    Key(KeyCode),
    Axis(ThresholdedAxis),
}

pub struct ModifierMap {
    modifier_mapping: HashMap<ModifierInput, Action>,
    axis_thresholds: AllAxisThresholds,
    axis_states: HashMap<AbsAxisCode, Option<ThresholdType>>,
}

impl ModifierMap {
    fn handle_key(&mut self, ev: &KeyEvent) -> Option<(Action, Option<Action>)> {
        self.modifier_mapping
            .get(&ModifierInput::Key(ev.key()))
            .cloned()
            .map(|act| (act, None))
    }

    fn handle_axis(&mut self, ev: &AbsAxisEvent) -> Option<(Action, Option<Action>)> {
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
    }
    pub fn handle_event(&mut self, ev: &InputEvent) -> Option<(Action, Option<Action>)> {
        match ev {
            InputEvent::KeyEvent(kev) => self.handle_key(kev),
            InputEvent::AbsAxisEvent(aev) => self.handle_axis(aev),
            InputEvent::RelAxisEvent(_) => None,
        }
    }

    /*
    pub fn peek_actions(&self) -> impl Iterator<Item = &Action> {
        self.modifier_mapping.values()
    }
    */

    pub fn peek_actions_mut(&mut self) -> impl Iterator<Item = &mut Action> {
        self.modifier_mapping.values_mut()
    }

    pub fn init(modifier_map: ModifierMapInput, all_a_t: AllAxisThresholds) -> Self {
        let mut modifier_mapping =
            HashMap::<ModifierInput, Action>::with_capacity(modifier_map.len());
        let mut axis_states = HashMap::<AbsAxisCode, Option<ThresholdType>>::new();
        for (input, output) in modifier_map {
            modifier_mapping.insert(input, output);
            if let ModifierInput::Axis(ta) = input {
                axis_states.insert(ta.code(), None);
            }
        }
        Self {
            modifier_mapping,
            axis_thresholds: all_a_t,
            axis_states,
        }
    }
}

impl From<KeyCode> for ModifierInput {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

impl From<ThresholdedAxis> for ModifierInput {
    fn from(t_a: ThresholdedAxis) -> Self {
        Self::Axis(t_a)
    }
}

impl From<(AbsAxisCode, ThresholdType)> for ModifierInput {
    fn from(data: (AbsAxisCode, ThresholdType)) -> Self {
        Self::Axis(data.into())
    }
}
