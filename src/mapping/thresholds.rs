use crate::constants::*;
use crate::events::AbsAxisEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An axis type that treats Absolute Axis input as digital input by using a threshold boundary.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ThresholdedAxis {
    axis: AbsAxisCode,
    threshold: ThresholdType,
}

impl ThresholdedAxis {
    /// Returns both possible thresholds for the AbsAxisEvent
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::events::*;
    /// use chord2key::constants::*;
    ///
    /// let t_axis = ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Greater);
    /// let t_axis_opp = t_axis.opposite();
    ///
    /// let (p1, p2) = ThresholdedAxis::all_possible(&AbsAxisEvent::new(AbsAxisCode::ABS_X, 2000));
    /// assert!(p1 != p2);
    /// assert!(p1 == t_axis || p1 == t_axis_opp);
    /// assert!(p2 == t_axis || p2 == t_axis_opp);
    /// ```
    pub fn all_possible(ev: &AbsAxisEvent) -> (Self, Self) {
        (
            Self {
                axis: ev.axis(),
                threshold: ThresholdType::Greater,
            },
            Self {
                axis: ev.axis(),
                threshold: ThresholdType::Lesser,
            },
        )
    }

    /// Returns a new ThresholdedAxis
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    ///
    /// let t_axis = ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Greater);
    /// ```
    pub fn new(axis: AbsAxisCode, threshold: ThresholdType) -> Self {
        Self { axis, threshold }
    }

    /// Returns the ThresholdedAxis with the opposite threshold direction
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    ///
    /// let t_axis = ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Greater);
    /// let t_axis_opp = t_axis.opposite();
    /// assert_eq!(t_axis_opp.threshold(), ThresholdType::Lesser);
    /// ```
    pub fn opposite(&self) -> Self {
        Self {
            axis: self.axis,
            threshold: self.threshold.opposite(),
        }
    }

    /// Returns the threshold type
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    ///
    /// let t_axis = ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Greater);
    /// assert_eq!(t_axis.threshold(), ThresholdType::Greater);
    /// ```
    pub fn threshold(&self) -> ThresholdType {
        self.threshold
    }

    /// Returns the axis code
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    ///
    /// let t_axis = ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Greater);
    /// assert_eq!(t_axis.code(), AbsAxisCode::ABS_X);
    /// ```
    pub fn code(&self) -> AbsAxisCode {
        self.axis
    }
}

/// The two possible threshold boundaries for a [ThresholdedAxis].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ThresholdType {
    Greater,
    Lesser,
}

impl ThresholdType {
    /// Returns the opposite threshold to this one
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// assert_eq!(ThresholdType::Greater.opposite(), ThresholdType::Lesser);
    /// assert_eq!(ThresholdType::Lesser.opposite(), ThresholdType::Greater);
    /// ```
    pub fn opposite(&self) -> Self {
        match self {
            Self::Greater => Self::Lesser,
            Self::Lesser => Self::Greater,
        }
    }
}

/// A threshold for an Absolute Axis in one direction
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AxisThreshold {
    /// The direction for the axis to surpass the threshold in.
    pub dir: ThresholdType,
    /// The value of the axis at the threshold.
    pub threshold: AxisState,
}

impl AxisThreshold {
    /// Checks if an AbsAxisEvent state passes this axis threshold
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::events::*;
    /// use chord2key::constants::*;
    ///
    /// let t = AxisThreshold {
    ///     dir: ThresholdType::Greater,
    ///     threshold: 2000,
    /// };
    ///
    /// let passing_event = AbsAxisEvent::new(AbsAxisCode::ABS_X, 2005);
    /// let failing_event = AbsAxisEvent::new(AbsAxisCode::ABS_X, 1995);
    ///
    /// assert!(t.is_passing(&passing_event));
    /// assert!(!t.is_passing(&failing_event));
    /// ```
    pub fn is_passing(&self, axis_event: &AbsAxisEvent) -> bool {
        let state = axis_event.state();

        (self.dir == ThresholdType::Greater && state >= self.threshold)
            || (self.dir == ThresholdType::Lesser && state <= self.threshold)
    }

    /// Changes self to have the lesser magnitude threshold, returning true if successful.
    ///
    /// Returns false if the two thresholds are in opposite directions
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    ///
    /// let mut t_strict = AxisThreshold {
    ///     dir: ThresholdType::Greater,
    ///     threshold: 2000,
    /// };
    ///
    /// let mut t_loose = AxisThreshold {
    ///     dir: ThresholdType::Greater,
    ///     threshold: 1000,
    /// };
    ///
    /// assert!(t_strict.loose_match(&t_loose));
    /// assert!(t_strict.threshold == 1000);
    ///
    /// let mut t_looser = AxisThreshold {
    ///     dir: ThresholdType::Greater,
    ///     threshold: 500,
    /// };
    ///
    /// assert!(t_looser.loose_match(&t_strict));
    /// assert!(t_looser.threshold == 500);
    ///
    /// let mut t_opp = AxisThreshold {
    ///     dir: ThresholdType::Lesser,
    ///     threshold: 1000,
    /// };
    ///
    /// assert!(!t_opp.loose_match(&t_looser));
    /// assert!(t_opp.threshold == 1000);
    ///```
    pub fn loose_match(&mut self, other: &AxisThreshold) -> bool {
        if self.dir == other.dir {
            match self.dir {
                ThresholdType::Greater => {
                    self.threshold = if self.threshold >= other.threshold {
                        other.threshold
                    } else {
                        self.threshold
                    };
                }
                ThresholdType::Lesser => {
                    self.threshold = if self.threshold <= other.threshold {
                        other.threshold
                    } else {
                        self.threshold
                    };
                }
            }
            return true;
        }
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AxisThresholds {
    first: AxisThreshold,
    second: Option<AxisThreshold>,
}

impl AxisThresholds {
    pub fn new(threshold: AxisThreshold) -> Self {
        Self {
            first: threshold,
            second: None,
        }
    }

    pub fn get_passing(&self, axis_event: &AbsAxisEvent) -> Option<ThresholdedAxis> {
        if self.first.is_passing(axis_event) {
            return Some(ThresholdedAxis {
                axis: axis_event.axis(),
                threshold: self.first.dir,
            });
        }
        if let Some(second) = self.second {
            if second.is_passing(axis_event) {
                return Some(ThresholdedAxis {
                    axis: axis_event.axis(),
                    threshold: second.dir,
                });
            }
        }
        None
    }

    pub fn get_passing_with_state(
        &self,
        axis_event: &AbsAxisEvent,
    ) -> Option<(ThresholdedAxis, AxisState)> {
        if self.first.is_passing(axis_event) {
            return Some((
                ThresholdedAxis {
                    axis: axis_event.axis(),
                    threshold: self.first.dir,
                },
                self.first.threshold,
            ));
        }
        if let Some(second) = self.second {
            if second.is_passing(axis_event) {
                return Some((
                    ThresholdedAxis {
                        axis: axis_event.axis(),
                        threshold: second.dir,
                    },
                    second.threshold,
                ));
            }
        }
        None
    }

    /// Attempts to add a threshold to this, returning true if compatible
    ///
    /// If the threshold is compatible, it will add it as-is if it's in a new direction. If it's in
    /// the same direction, the looser threshold will be chosen.
    pub fn loose_add(&mut self, threshold: AxisThreshold) {
        if !self.first.loose_match(&threshold) {
            if let Some(second) = self.second.as_mut() {
                second.loose_match(&threshold);
            } else {
                self.second = Some(threshold);
            }
        }
    }
}

/// A structure holding all of the configured axis thresholds
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllAxisThresholds {
    map: HashMap<AbsAxisCode, AxisThresholds>,
}

impl AllAxisThresholds {
    /// Creates a new AllAxisThresholds
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    ///
    /// let a_a_t = AllAxisThresholds::init(vec![
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Greater, threshold: 2000}),
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Lesser, threshold: -2000}),
    /// ]);
    /// ```
    pub fn init(thresholds: Vec<(AbsAxisCode, AxisThreshold)>) -> Self {
        let mut map = HashMap::<AbsAxisCode, AxisThresholds>::new();
        for (code, threshold) in thresholds {
            if let Some(stored) = map.get_mut(&code) {
                stored.loose_add(threshold);
            } else {
                map.insert(code, AxisThresholds::new(threshold));
            }
        }

        Self { map }
    }

    /// Returns the passing [ThresholdedAxis] input from the [AbsAxisEvent], if any
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    ///
    /// let a_a_t = AllAxisThresholds::init(vec![
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Greater, threshold: 2000}),
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Lesser, threshold: -2000}),
    /// ]);
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing(&AbsAxisEvent::new(AbsAxisCode::ABS_X, 4000)),
    ///     Some(ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Greater))
    /// );
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing(&AbsAxisEvent::new(AbsAxisCode::ABS_X, -4000)),
    ///     Some(ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Lesser))
    /// );
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing(&AbsAxisEvent::new(AbsAxisCode::ABS_X, 0)),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing(&AbsAxisEvent::new(AbsAxisCode::ABS_Y, 4000)),
    ///     None
    /// );
    /// ```
    pub fn get_passing(&self, axis_event: &AbsAxisEvent) -> Option<ThresholdedAxis> {
        self.map
            .get(&axis_event.axis())
            .map(|thresholds| thresholds.get_passing(axis_event))
            .flatten()
    }

    /// Returns the passing [ThresholdedAxis] input from the [AbsAxisEvent], if any, along with the
    /// threshold [AxisState] value.
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    ///
    /// let a_a_t = AllAxisThresholds::init(vec![
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Greater, threshold: 2000}),
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Lesser, threshold: -2000}),
    /// ]);
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing_with_state(&AbsAxisEvent::new(AbsAxisCode::ABS_X, 4000)),
    ///     Some((ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Greater), 2000))
    /// );
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing_with_state(&AbsAxisEvent::new(AbsAxisCode::ABS_X, -4000)),
    ///     Some((ThresholdedAxis::new(AbsAxisCode::ABS_X, ThresholdType::Lesser), -2000))
    /// );
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing_with_state(&AbsAxisEvent::new(AbsAxisCode::ABS_X, 0)),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     a_a_t.get_passing_with_state(&AbsAxisEvent::new(AbsAxisCode::ABS_Y, 4000)),
    ///     None
    /// );
    /// ```
    pub fn get_passing_with_state(
        &self,
        axis_event: &AbsAxisEvent,
    ) -> Option<(ThresholdedAxis, AxisState)> {
        self.map
            .get(&axis_event.axis())
            .map(|thresholds| thresholds.get_passing_with_state(axis_event))
            .flatten()
    }

    /// Returns if the axis within the [AbsAxisEvent] has a stored threshold
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    ///
    /// let a_a_t = AllAxisThresholds::init(vec![
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Greater, threshold: 2000}),
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Lesser, threshold: -2000}),
    /// ]);
    ///
    /// assert!(a_a_t.has_threshold(&AbsAxisEvent::new(AbsAxisCode::ABS_X, 4000)));
    /// assert!(!a_a_t.has_threshold(&AbsAxisEvent::new(AbsAxisCode::ABS_Y, 4000)));
    /// ```
    pub fn has_threshold(&self, axis_event: &AbsAxisEvent) -> bool {
        self.map.contains_key(&axis_event.axis())
    }

    /// Returns an iterator over all the AbsAxisCodes that have a threshold
    ///
    /// Example:
    /// ```
    /// use chord2key::mapping::thresholds::*;
    /// use chord2key::constants::*;
    /// use chord2key::events::*;
    ///
    /// let a_a_t = AllAxisThresholds::init(vec![
    ///     (AbsAxisCode::ABS_X, AxisThreshold{dir: ThresholdType::Greater, threshold: 2000}),
    ///     (AbsAxisCode::ABS_Y, AxisThreshold{dir: ThresholdType::Lesser, threshold: -2000}),
    ///     (AbsAxisCode::ABS_RX, AxisThreshold{dir: ThresholdType::Greater, threshold: 1000}),
    /// ]);
    ///
    /// let mut codes = vec![&AbsAxisCode::ABS_X, &AbsAxisCode::ABS_Y, &AbsAxisCode::ABS_RX];
    /// let mut test_codes: Vec<&AbsAxisCode> = a_a_t.codes().collect();
    ///
    /// codes.sort();
    /// test_codes.sort();
    ///
    /// assert_eq!(test_codes, codes);
    /// ```
    pub fn codes(&self) -> impl Iterator<Item = &AbsAxisCode> {
        self.map.iter().map(|(key, _)| key)
    }
}

impl From<(AbsAxisCode, ThresholdType)> for ThresholdedAxis {
    fn from(data: (AbsAxisCode, ThresholdType)) -> Self {
        Self::new(data.0, data.1)
    }
}

impl From<(ThresholdType, AxisState)> for AxisThreshold {
    fn from(data: (ThresholdType, AxisState)) -> Self {
        Self {
            dir: data.0,
            threshold: data.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axis_thresholds_loose_add() {
        let threshold1 = AxisThreshold {
            dir: ThresholdType::Greater,
            threshold: 30,
        };
        let mut thresholds = AxisThresholds {
            first: threshold1,
            second: None,
        };

        assert_eq!(thresholds.first.threshold, 30);

        thresholds.loose_add(AxisThreshold {
            dir: ThresholdType::Greater,
            threshold: 15,
        });

        assert_eq!(thresholds.first.threshold, 15);

        thresholds.loose_add(AxisThreshold {
            dir: ThresholdType::Lesser,
            threshold: -30,
        });

        assert_eq!(thresholds.first.threshold, 15);
        assert_eq!(thresholds.second.unwrap().threshold, -30);

        thresholds.loose_add(AxisThreshold {
            dir: ThresholdType::Lesser,
            threshold: -15,
        });

        assert_eq!(thresholds.first.threshold, 15);
        assert_eq!(thresholds.second.unwrap().threshold, -15);

        thresholds.loose_add(AxisThreshold {
            dir: ThresholdType::Lesser,
            threshold: -45,
        });

        assert_eq!(thresholds.first.threshold, 15);
        assert_eq!(thresholds.second.unwrap().threshold, -15);
    }
}
