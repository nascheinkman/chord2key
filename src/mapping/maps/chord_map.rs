use crate::attribute_set::*;
use crate::constants::*;
use crate::events::*;
use crate::input::events::InputEvent;
use crate::mapping::actions::*;
use crate::mapping::thresholds::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::matches;
use std::rc::Rc;

/// A list of tuples that map a list of [ChordInput] to an [Action].
pub type ChordMapInput = Vec<(Vec<ChordInput>, Action)>;

/// The type used to represent a chord.
pub type Chord = AttributeSubset<ChordInput>;

/// The input types used to create a chord.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ChordInput {
    Key(KeyCode),
    ThresholdedAxis(ThresholdedAxis),
}

pub struct ChordMap {
    chord_inputs: Rc<AttributeSet<ChordInput>>,
    axis_thresholds: AllAxisThresholds,
    chord_mapping: HashMap<Chord, Action>,
    primed: bool,
    state: Chord,
    prev_chord: Chord,
}

impl ChordMap {
    #[allow(clippy::ptr_arg)]
    fn construct_input_set(chord_map: &ChordMapInput) -> Rc<AttributeSet<ChordInput>> {
        let mut input_hash_set = HashSet::<ChordInput>::new();
        chord_map
            .iter()
            .map(|(chord, _action)| chord)
            .for_each(|chord| {
                chord.iter().for_each(|input| {
                    input_hash_set.insert(*input);
                })
            });

        let size = input_hash_set.len();
        AttributeSet::<ChordInput>::from_capacity(input_hash_set, size)
    }

    fn map_chord(&mut self, chord: Vec<ChordInput>, action: Action) {
        self.chord_mapping
            .insert(self.chord_inputs.subset_with(chord), action);
    }

    fn fill_chords(&mut self, chord_map: ChordMapInput) {
        for (chord_vec, action) in chord_map {
            self.map_chord(chord_vec, action);
        }
    }

    /*
    pub fn peek_actions(&self) -> impl Iterator<Item = &Action> {
        self.chord_mapping.values()
    }
    */

    pub fn peek_actions_mut(&mut self) -> impl Iterator<Item = &mut Action> {
        self.chord_mapping.values_mut()
    }

    fn get_action(&self, chord: &Chord) -> Option<Action> {
        self.chord_mapping.get(chord).cloned()
    }

    pub fn get_prev_action(&self) -> Option<Action> {
        self.chord_mapping.get(&self.prev_chord).cloned()
    }

    fn emit_action(&mut self) -> Option<Action> {
        let mut action: Option<Action> = None;

        // If ready to emit an action
        if self.primed {
            // See if the chord results in an action
            action = self.get_action(&self.state);
            if let Some(ref action) = action {
                // Update previous chord for chord repition
                if !matches!(
                    action,
                    &Action::InnerAction(InnerAction::RepeatLastChord { .. })
                ) {
                    self.prev_chord.copy_from(&self.state).ok();
                }

                // No longer ready to emit actions
                self.primed = false;
            }
        }

        action
    }

    fn handle_key(&mut self, ev: &KeyEvent) -> Option<Action> {
        let key = &ChordInput::Key(ev.key());

        match ev.state() {
            PressState::Down => {
                if self.state.try_insert(key).is_ok() {
                    self.primed = true;
                }
                None
            }
            PressState::Up => {
                let action = self.emit_action();

                self.state.remove(key);

                action
            }
        }
    }

    fn handle_axis(&mut self, ev: &AbsAxisEvent) -> Option<Action> {
        let (possible1, possible2) = ThresholdedAxis::all_possible(ev);
        let possible1: &ChordInput = &possible1.into();
        let possible2: &ChordInput = &possible2.into();

        if !(self.chord_inputs.contains(possible1) || self.chord_inputs.contains(possible2)) {
            return None;
        }
        let passing_t = self.axis_thresholds.get_passing(ev);

        let mut action: Option<Action> = None;
        match passing_t {
            Some(passing_t) => {
                if self.state.contains(&passing_t.into()) {
                    return None;
                }
                if self.state.contains(possible1) || self.state.contains(possible2) {
                    action = self.emit_action();
                    self.state.remove(&passing_t.opposite().into());
                }
                self.primed = true;
                self.state.try_insert(&passing_t.into()).ok();
            }
            None => {
                if self.state.contains(possible1) {
                    action = self.emit_action();
                    self.state.remove(possible1);
                }
                if self.state.contains(possible2) {
                    action = self.emit_action();
                    self.state.remove(possible2);
                }
            }
        }

        action
    }

    pub fn clear_state(&mut self) {
        self.state.clear();
    }

    pub fn handle_event(&mut self, ev: &InputEvent) -> Option<Action> {
        match ev {
            InputEvent::KeyEvent(kev) => self.handle_key(kev),
            InputEvent::AbsAxisEvent(aev) => self.handle_axis(aev),
            InputEvent::RelAxisEvent(_) => None,
        }
    }

    pub fn init(chord_map: ChordMapInput, all_a_t: AllAxisThresholds) -> Self {
        let chord_inputs = ChordMap::construct_input_set(&chord_map);
        let chord_mapping = HashMap::<Chord, Action>::with_capacity(chord_map.len());

        let state = chord_inputs.empty_subset();
        let prev_chord = chord_inputs.empty_subset();

        let mut new_self = Self {
            chord_inputs,
            axis_thresholds: all_a_t,
            chord_mapping,
            primed: true,
            state,
            prev_chord,
        };
        new_self.fill_chords(chord_map);

        new_self
    }
}

impl From<KeyCode> for ChordInput {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

impl From<ThresholdedAxis> for ChordInput {
    fn from(t_a: ThresholdedAxis) -> Self {
        Self::ThresholdedAxis(t_a)
    }
}

impl From<(AbsAxisCode, ThresholdType)> for ChordInput {
    fn from(data: (AbsAxisCode, ThresholdType)) -> Self {
        Self::ThresholdedAxis(data.into())
    }
}
