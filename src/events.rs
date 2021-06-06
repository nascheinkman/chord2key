use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The relevant data involved in a Key event.
///
/// A lot of extra information is passed in an evdev::InputEvent, so this structure simplifies the
/// data to only the relevant information in a key event: the key and its state.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    key: KeyCode,
    state: PressState,
}

impl KeyEvent {
    /// Creates a new KeyEvent with the specified key and state.
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev1 = KeyEvent::new(KeyCode::KEY_A, PressState::Up);
    /// let ev2 = KeyEvent::new(KeyCode::KEY_A, PressState::Down);
    /// let ev3 = KeyEvent::new(KeyCode::KEY_A, PressState::Up);
    /// assert!(ev1 != ev2);
    /// assert!(ev1 == ev3);
    /// ```
    pub fn new(key: KeyCode, state: PressState) -> Self {
        Self {
            key: key,
            state: state,
        }
    }

    /// Retrieves the KeyCode for the event.
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev = KeyEvent::new(KeyCode::KEY_W, PressState::Down);     
    /// assert_eq!(ev.key(), KeyCode::KEY_W);
    /// ```

    pub fn key(&self) -> KeyCode {
        self.key
    }

    /// Retrieves the state of the key reported by the event.
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev = KeyEvent::new(KeyCode::KEY_W, PressState::Down);     
    /// assert_eq!(ev.state(), PressState::Down);
    /// ```
    pub fn state(&self) -> PressState {
        self.state
    }
}

impl TryFrom<evdev::InputEvent> for KeyEvent {
    type Error = &'static str;
    fn try_from(ev: evdev::InputEvent) -> Result<Self, Self::Error> {
        let error: &'static str = "Could not convert evdev::InputEvent into KeyEvent";

        // If the event is a key event
        if let evdev::InputEventKind::Key(code) = ev.kind() {
            // Match the key
            let key = match num::FromPrimitive::from_u16(code.0) {
                Some(key) => key,
                None => return Err(error),
            };

            // Set the PressState
            match ev.value() {
                1 => {
                    return Ok(Self {
                        key: key,
                        state: PressState::Down,
                    })
                }
                0 => {
                    return Ok(Self {
                        key: key,
                        state: PressState::Up,
                    })
                }
                _ => return Err(error),
            }
        }
        Err(error)
    }
}

/// The relevant data involved in a Relative Axis event.
///
/// A lot of extra information is passed in an evdev::InputEvent, so this structure simplifies the
/// data to only the relevant information in a Relative Axis event: the axis and its value.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RelAxisEvent {
    axis: RelAxisCode,
    state: AxisState,
}

impl RelAxisEvent {
    /// Creates a new RelAxisEvent with the specified axis and state.
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev1 = RelAxisEvent::new(RelAxisCode::REL_X, -5);
    /// let ev2 = RelAxisEvent::new(RelAxisCode::REL_X, 5);
    /// let ev3 = RelAxisEvent::new(RelAxisCode::REL_X, -5);
    /// assert!(ev1 != ev2);
    /// assert!(ev1 == ev3);
    /// ```
    pub fn new(axis: RelAxisCode, val: AxisState) -> Self {
        Self {
            axis: axis,
            state: val,
        }
    }

    /// Retrieves the axis for the event.
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev = RelAxisEvent::new(RelAxisCode::REL_X, -5);
    /// assert_eq!(ev.axis(), RelAxisCode::REL_X);
    /// ```                         
    pub fn axis(&self) -> RelAxisCode {
        self.axis
    }

    /// Retrieves the value of the axis for the event
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev = RelAxisEvent::new(RelAxisCode::REL_X, -5);
    /// assert_eq!(ev.state(), -5);
    /// ```
    pub fn state(&self) -> AxisState {
        self.state
    }

    /// Sets the state of the axis for the event
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let mut ev = RelAxisEvent::new(RelAxisCode::REL_X, -5);
    /// assert_eq!(ev.state(), -5);
    /// ev.set_state(10);
    /// assert_eq!(ev.state(), 10);
    /// ```
    pub fn set_state(&mut self, state: AxisState) {
        self.state = state;
    }
}

impl TryFrom<evdev::InputEvent> for RelAxisEvent {
    type Error = &'static str;
    fn try_from(ev: evdev::InputEvent) -> Result<Self, Self::Error> {
        let error: &'static str = "Could not convert evdev::InputEvent into RelAxisEvent";
        if let evdev::InputEventKind::RelAxis(code) = ev.kind() {
            let axis = match num::FromPrimitive::from_u16(code.0) {
                Some(axis) => axis,
                None => return Err(error),
            };
            return Ok(Self {
                axis: axis,
                state: ev.value(),
            });
        }
        Err(error)
    }
}

/// The relevant data involved in a Absolute Axis event.
///
/// A lot of extra information is passed in an evdev::InputEvent, so this structure simplifies the
/// data to only the relevant information: the axis and its value.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AbsAxisEvent {
    axis: AbsAxisCode,
    state: AxisState,
}
impl AbsAxisEvent {
    /// Creates a new AbsAxisEvent with the specified axis and state.
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev1 = AbsAxisEvent::new(AbsAxisCode::ABS_X, -5);
    /// let ev2 = AbsAxisEvent::new(AbsAxisCode::ABS_X, 5);
    /// let ev3 = AbsAxisEvent::new(AbsAxisCode::ABS_X, -5);
    /// assert!(ev1 != ev2);
    /// assert!(ev1 == ev3);
    /// ```
    pub fn new(axis: AbsAxisCode, state: AxisState) -> Self {
        Self {
            axis: axis,
            state: state,
        }
    }

    /// Retrieves the axis for the event.
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev = AbsAxisEvent::new(AbsAxisCode::ABS_X, -5);
    /// assert_eq!(ev.axis(), AbsAxisCode::ABS_X);
    /// ```
    pub fn axis(&self) -> AbsAxisCode {
        self.axis
    }

    /// Retrieves the value of the axis for the event
    ///
    /// Example:
    /// ```
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    /// let ev = AbsAxisEvent::new(AbsAxisCode::ABS_X, -5);
    /// assert_eq!(ev.state(), -5);
    /// ```
    pub fn state(&self) -> AxisState {
        self.state
    }
}

impl TryFrom<evdev::InputEvent> for AbsAxisEvent {
    type Error = &'static str;
    fn try_from(ev: evdev::InputEvent) -> Result<Self, Self::Error> {
        let error: &'static str = "Could not convert evdev::InputEvent into AbsAxisEvent";
        if let evdev::InputEventKind::AbsAxis(code) = ev.kind() {
            let axis = match num::FromPrimitive::from_u16(code.0) {
                Some(axis) => axis,
                None => return Err(error),
            };
            return Ok(Self {
                axis: axis,
                state: ev.value(),
            });
        }
        Err(error)
    }
}
