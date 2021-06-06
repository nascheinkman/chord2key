use crate::events::*;
use std::convert::TryFrom;

/// The different types of events used by this crate.
///
/// Lots of other event types exist, such as toggling switches on the device, but only events
/// thought relevant to user input were used.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum InputEvent {
    KeyEvent(KeyEvent),
    RelAxisEvent(RelAxisEvent),
    AbsAxisEvent(AbsAxisEvent),
}

impl TryFrom<evdev::InputEvent> for InputEvent {
    type Error = &'static str;
    fn try_from(ev: evdev::InputEvent) -> Result<Self, Self::Error> {
        let error: &'static str = "Could not convert evdev::InputEvent into chord2key::InputEvent";
        if let Some(ev_key) = KeyEvent::try_from(ev).ok() {
            return Ok(Self::KeyEvent(ev_key));
        }
        if let Some(ev_abs) = AbsAxisEvent::try_from(ev).ok() {
            return Ok(Self::AbsAxisEvent(ev_abs));
        }
        if let Some(ev_rel) = RelAxisEvent::try_from(ev).ok() {
            return Ok(Self::RelAxisEvent(ev_rel));
        }
        Err(error)
    }
}
