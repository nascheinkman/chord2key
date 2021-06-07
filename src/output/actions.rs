use crate::constants::*;
pub use crate::events::RelAxisEvent;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

/// An OutputAction is an action that can be sent to the OutputDevice to simulate keyboard and mouse
/// inputs.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum OutputAction {
    StateChange(StateChange),
    Pulse(Pulse),
    Toggle(Toggle),
}

/// The different types of possible OutputActions
///
/// Used for when wanting to denote the type of an OutputAction without having the actual event
/// struct.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum OutputActionType {
    StateChange,
    Pulse,
    Toggle,
}

/// A Toggle output event.
///
/// Equivalent to pressing the key if it's not pressed, or letting go of a key if it is pressed.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Toggle {
    pub keys: Option<KeyList>,
    pub axes: Option<AxisList>,
}

impl Toggle {
    /// Create a new Toggle event.
    ///
    /// Example:
    /// ```
    /// use chord2key::output::actions::*;
    /// use chord2key::constants::*;
    ///
    /// let t = Toggle::new(
    ///     Some(vec![KeyCode::KEY_W]),
    ///     Some(vec![(RelAxisCode::REL_X, 5)].into()),
    /// );
    /// ```
    pub fn new(keys: Option<KeyList>, axes: Option<AxisList>) -> Self {
        Self {
            keys,
            axes,
        }
    }
}

/// The data representing a pulsed output event
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Pulse {
    pub keys: Option<KeyList>,
    pub axes: Option<AxisList>,
}

impl Pulse {
    /// Creates a new Pulse
    ///
    /// # Example
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::output::actions::*;
    ///
    /// let pulse = Pulse::new(Some(vec![KeyCode::KEY_W]), None);
    ///
    /// assert!(pulse.keys.is_some());
    /// assert!(pulse.axes.is_none());
    /// ```
    pub fn new(keys: Option<KeyList>, axes: Option<AxisList>) -> Self {
        Self {
            keys,
            axes,
        }
    }
}

/// The data used to represent a change of output state
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct StateChange {
    pub keys: Option<KeyStateChange>,
    pub axes: Option<AxisList>,
}

impl StateChange {
    /// Creates a new StateChange
    ///
    /// # Example
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::output::actions::*;
    ///
    /// let keys = Some(KeyStateChange {
    ///     keys: vec![KeyCode::KEY_W],
    ///     state: PressState::Down,
    /// });
    /// let change = StateChange::new(keys, None);
    ///
    /// assert!(change.axes.is_none());
    /// assert!(change.keys.unwrap().keys[0] == KeyCode::KEY_W);
    /// ```
    pub fn new(keys: Option<KeyStateChange>, axes: Option<AxisList>) -> Self {
        Self {
            keys,
            axes,
        }
    }

    /// Inverts a StateChange to its reciprocal.
    ///
    /// This flips the press state of its keys (up to down and vice versa), and sets all axis values
    /// to 0.
    ///
    /// # Example
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::output::actions::*;
    ///
    /// let keys = Some(KeyStateChange {
    ///     keys: vec![KeyCode::KEY_W],
    ///     state: PressState::Down,
    /// });
    /// let axes = Some(vec![(RelAxisCode::REL_X, 5)].into());
    /// let mut change = StateChange::new(keys, axes);
    ///
    /// if let Some(ref axes) = change.axes {
    ///     assert!(axes.iter().next().unwrap().state() == 5);
    /// }
    /// if let Some(ref keys) = change.keys {
    ///     assert!(keys.state == PressState::Down);
    /// }
    ///
    /// change.inverse();
    ///
    /// assert!(change.axes.unwrap().iter().next().unwrap().state() == 0);
    /// assert!(change.keys.unwrap().state == PressState::Up);
    /// ```
    pub fn inverse(&mut self) -> &mut Self {
        self.keys.iter_mut().for_each(|change| change.inverse());
        self.axes.iter_mut().for_each(|change| change.inverse());
        self
    }
}

/// The data used to represent a change of key state
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct KeyStateChange {
    pub keys: KeyList,
    pub state: PressState,
}

impl KeyStateChange {
    /// Inverses the state of a KeyStateChange
    ///
    /// # Example
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::output::actions::*;
    ///
    /// let mut change = KeyStateChange {
    ///     keys: vec![KeyCode::KEY_W, KeyCode::KEY_S],
    ///     state: PressState::Down,
    /// };
    ///
    /// assert_eq!(change.state, PressState::Down);
    ///
    /// change.inverse();
    /// assert_eq!(change.state, PressState::Up);
    /// ```
    pub fn inverse(&mut self) {
        match self.state {
            PressState::Down => {
                self.state = PressState::Up;
            }
            PressState::Up => {
                self.state = PressState::Down;
            }
        }
    }
}

impl From<StateChange> for Pulse {
    fn from(sc: StateChange) -> Self {
        Self {
            keys: sc.keys.map(|ksc| ksc.keys),
            axes: sc.axes,
        }
    }
}
impl From<Toggle> for Pulse {
    fn from(t: Toggle) -> Self {
        Self {
            keys: t.keys,
            axes: t.axes,
        }
    }
}

impl From<Pulse> for StateChange {
    fn from(pulse: Pulse) -> Self {
        Self {
            keys: pulse.keys.map(|keys| KeyStateChange {
                keys,
                state: PressState::Down,
            }),
            axes: pulse.axes,
        }
    }
}
impl From<Toggle> for StateChange {
    fn from(t: Toggle) -> Self {
        Self {
            keys: t.keys.map(|keys| KeyStateChange {
                keys,
                state: PressState::Down,
            }),
            axes: t.axes,
        }
    }
}

impl From<Pulse> for Toggle {
    fn from(pulse: Pulse) -> Self {
        Self {
            keys: pulse.keys,
            axes: pulse.axes,
        }
    }
}
impl From<StateChange> for Toggle {
    fn from(sc: StateChange) -> Self {
        Self {
            keys: sc.keys.map(|ksc| ksc.keys),
            axes: sc.axes,
        }
    }
}

impl From<OutputAction> for Pulse {
    fn from(ev: OutputAction) -> Self {
        match ev {
            OutputAction::Pulse(p) => p,
            OutputAction::StateChange(sc) => sc.into(),
            OutputAction::Toggle(t) => t.into(),
        }
    }
}
impl From<OutputAction> for StateChange {
    fn from(ev: OutputAction) -> Self {
        match ev {
            OutputAction::Pulse(p) => p.into(),
            OutputAction::StateChange(sc) => sc,
            OutputAction::Toggle(t) => t.into(),
        }
    }
}
impl From<OutputAction> for Toggle {
    fn from(ev: OutputAction) -> Self {
        match ev {
            OutputAction::Pulse(p) => p.into(),
            OutputAction::StateChange(sc) => sc.into(),
            OutputAction::Toggle(t) => t,
        }
    }
}

/// A list of KeyCodes.
pub type KeyList = Vec<KeyCode>;

/// A list of Relative Axes and their state
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AxisList(Vec<RelAxisEvent>);

impl AxisList {
    /// Inverts all axis states inside the list
    ///
    /// The inverse of an axis state is defined to always be 0.
    ///
    /// Example:
    /// ```
    /// use chord2key::output::actions::*;
    /// use chord2key::constants::*;
    ///
    /// let mut list: AxisList = vec![
    ///     (RelAxisCode::REL_X, -5),
    ///     (RelAxisCode::REL_Y,-5),
    /// ].into();
    ///
    /// list.iter().for_each(|e| {
    ///     assert!(e.state() == -5);
    /// });
    ///
    /// list.inverse();
    ///
    /// list.iter().for_each(|e| {
    ///     assert!(e.state() == 0);
    /// });
    pub fn inverse(&mut self) {
        self.0.iter_mut().for_each(|e| e.set_state(0));
    }

    /// Returns an iterator over immutable references to the RelAxisEvents within the list
    ///
    /// Example:
    /// ```
    /// use chord2key::output::actions::*;
    /// use chord2key::constants::*;
    ///
    /// let list: AxisList = vec![
    ///     (RelAxisCode::REL_X, -5),
    ///     (RelAxisCode::REL_Y,-5),
    /// ].into();
    ///
    /// list.iter().for_each(|e| {
    ///     assert!(e.state() == -5);
    /// });
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &RelAxisEvent> {
        self.0.iter()
    }

    /// Returns an iterator over mutable references to the RelAxisEvents within the list
    ///
    /// Example:
    /// ```
    /// use chord2key::output::actions::*;
    /// use chord2key::constants::*;
    ///
    /// let mut list: AxisList = vec![
    ///     (RelAxisCode::REL_X, -5),
    ///     (RelAxisCode::REL_Y,-5),
    /// ].into();
    ///
    /// list.iter().for_each(|e| {
    ///     assert!(e.state() == -5);
    /// });
    ///
    /// list.iter_mut().for_each(|e| {
    ///     e.set_state(2);
    /// });
    ///
    /// list.iter().for_each(|e| {
    ///     assert!(e.state() == 2);
    /// });
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RelAxisEvent> {
        self.0.iter_mut()
    }
}

impl FromIterator<RelAxisEvent> for AxisList {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RelAxisEvent>,
    {
        Self(Vec::<RelAxisEvent>::from_iter(iter))
    }
}

impl From<Vec<(RelAxisCode, i32)>> for AxisList {
    fn from(al: Vec<(RelAxisCode, i32)>) -> Self {
        Self(
            al.iter()
                .map(|(code, state)| RelAxisEvent::new(*code, *state))
                .collect(),
        )
    }
}
