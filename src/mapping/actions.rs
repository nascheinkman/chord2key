use crate::output::actions::*;
use serde::{Deserialize, Serialize};

/// User actions accepted by the chord2key [Mapper].
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Actions that are sent to the [OutputDevice].
    OutputAction(OutputAction),

    /// Actions that act on or depend on the internal state of the [Mapper].
    InnerAction(InnerAction),
}

/// Actions that act on or depend on the internal state of the [Mapper].
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InnerAction {
    /// Repeat the last emitted chord in whatever [OutputActionType], converting if necessary
    RepeatLastChord(OutputActionType),

    /// Switch to a new configuration given by the path.
    SwitchConfig(std::path::PathBuf),
}

impl From<OutputAction> for Action {
    fn from(oe: OutputAction) -> Self {
        Self::OutputAction(oe)
    }
}

impl From<Pulse> for Action {
    fn from(pulse: Pulse) -> Self {
        Self::OutputAction(OutputAction::Pulse(pulse))
    }
}

impl From<StateChange> for Action {
    fn from(sc: StateChange) -> Self {
        Self::OutputAction(OutputAction::StateChange(sc))
    }
}

impl From<Toggle> for Action {
    fn from(toggle: Toggle) -> Self {
        Self::OutputAction(OutputAction::Toggle(toggle))
    }
}

impl From<InnerAction> for Action {
    fn from(ie: InnerAction) -> Self {
        Self::InnerAction(ie)
    }
}
