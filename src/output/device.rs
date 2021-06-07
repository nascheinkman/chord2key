use crate::constants::*;
use crate::output::actions::*;
use crate::strum::IntoEnumIterator;
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use std::collections::HashMap;
use std::result::Result;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SendError, Sender};
use std::thread;
use std::time::Duration;

/// A virtual device used to send keyboard and mouse output events.
///
/// Creates a new thread to handle events. Events are still executed as soon as possible as they
/// come in, but the new thread allows for state changes on the relative axes. The state of the
/// axis represents its velocity, and that velocity input is continously sent to the OS in a regular
/// time interval reported by [OutputDevice::pulse_time].
///
/// The created thread has the same lifetime as the struct.
pub struct OutputDevice {
    event_tx: Sender<OutputAction>,
}

impl OutputDevice {
    /// The amount of time, in milliseconds, between mouse event pulses.
    ///
    /// If it's too slow, the mouse seems jerky. If it's too fast, there's no ability for fine
    /// control. Arbitrarily, 20 milliseconds was chosen.
    ///
    /// # Example
    /// ```
    /// use chord2key::output::device::OutputDevice;
    /// use std::time::Duration;
    /// assert_eq!(OutputDevice::pulse_time(), Duration::from_millis(20));
    /// ```
    pub const fn pulse_time() -> Duration {
        Duration::from_millis(20)
    }

    /// Creates a new output device
    ///
    /// Returns Ok([OutputDevice]) if successful. If it could not be created, it is likely that the
    /// program was not run with sufficient permissions.
    pub fn init() -> Result<Self, std::io::Error> {
        let mut output = VirtualOutput::init()?;
        let (tx, rx): (Sender<OutputAction>, Receiver<OutputAction>) = mpsc::channel();
        let _handle = thread::spawn(move || {
            let mut start = std::time::Instant::now();
            loop {
                let elapsed = start.elapsed();
                let diff = OutputDevice::pulse_time().checked_sub(elapsed);
                let diff = match diff {
                    Some(diff) => diff,
                    None => {
                        // Time to pulse the axes
                        output.pulse_rel_axes();

                        // Reset timer
                        start = std::time::Instant::now();
                        OutputDevice::pulse_time()
                    }
                };
                let event_res = rx.recv_timeout(diff);
                match event_res {
                    Ok(event) => {
                        output.execute_event(&event);
                    }
                    Err(e) => {
                        if e == RecvTimeoutError::Disconnected {
                            break;
                        }
                    }
                }
            }
        });
        Ok(Self { event_tx: tx })
    }

    /// Executes an [OutputAction], consuming it in the process
    ///
    /// Returns a result indicating whether the event was successfully sent
    pub fn execute_event(&self, event: OutputAction) -> Result<(), SendError<OutputAction>> {
        self.event_tx.send(event)
    }
}

impl Clone for OutputDevice {
    fn clone(&self) -> Self {
        Self {
            event_tx: self.event_tx.clone(),
        }
    }
}

/// A wrapper around the [evdev::uinput::VirtualDevice] that only exposes a simple API for use.
struct DeviceWrapper(VirtualDevice);

impl DeviceWrapper {
    /// Emit an event to the OS to press a key down.
    pub fn down_key(&mut self, key: KeyCode) {
        let evkey = evdev::Key::from(key);
        let down_event = evdev::InputEvent::new(evdev::EventType::KEY, evkey.code(), 1);
        self.0.emit(&[down_event]).unwrap();
    }

    /// Emit an event to the OS.
    pub fn up_key(&mut self, key: KeyCode) {
        let evkey = evdev::Key::from(key);
        let up_event = evdev::InputEvent::new(evdev::EventType::KEY, evkey.code(), 0);
        self.0.emit(&[up_event]).unwrap();
    }

    /// Emit an event to the OS to move a Relative Axis, such as those used in mouse input.
    pub fn rel_axis_move(&mut self, axis: RelAxisCode, value: AxisState) {
        let evaxis = evdev::RelativeAxisType::from(axis);
        let axis_event = evdev::InputEvent::new(evdev::EventType::RELATIVE, evaxis.0, value);
        self.0.emit(&[axis_event]).unwrap();
    }
}

/// Turns an [OutputAction] into relevant events for the OS. Saves an internal state.
struct VirtualOutput {
    /// The actual device emitting events.
    pub device: DeviceWrapper,

    /// A saved state of all current relative axis states.
    pub rel_axes_vals: HashMap<RelAxisCode, AxisState>,

    /// A saved state of all current key states.
    pub key_states: HashMap<KeyCode, PressState>,
}

impl VirtualOutput {
    /// Attempts to create a new VirtualOutput, returning a resulting containing itself.
    ///
    /// May fail due to lack of OS permissions.
    pub fn init() -> std::io::Result<Self> {
        let mut key_set = evdev::AttributeSet::<evdev::Key>::new();
        let mut rel_axes_vals = HashMap::<RelAxisCode, AxisState>::new();
        let mut key_states = HashMap::<KeyCode, PressState>::new();
        KeyCode::iter().for_each(|key| {
            key_set.insert(evdev::Key::from(key));
            key_states.insert(key, PressState::Up);
        });

        let mut axis_set = evdev::AttributeSet::<evdev::RelativeAxisType>::new();
        RelAxisCode::iter().for_each(|axis| {
            axis_set.insert(evdev::RelativeAxisType::from(axis));
            rel_axes_vals.insert(axis, 0);
        });

        let device = VirtualDeviceBuilder::new()?
            .name("chord2key Device")
            .with_keys(&key_set)?
            .with_relative_axes(&axis_set)?
            .build()?;

        Ok(Self {
            device: DeviceWrapper(device),
            rel_axes_vals,
            key_states,
        })
    }

    /// Emit a key press event and save the state of the key.
    fn down_key(&mut self, key: KeyCode) {
        self.device.down_key(key);
        self.key_states.insert(key, PressState::Down);
    }

    /// Emit a key unpress event and save the state of the key.
    fn up_key(&mut self, key: KeyCode) {
        self.device.up_key(key);
        self.key_states.insert(key, PressState::Up);
    }

    /// Emit a relative axis movement event, without saving the state of the axis.
    fn rel_axis_move(&mut self, axis: RelAxisCode, value: AxisState) {
        self.device.rel_axis_move(axis, value);
    }

    /// Emit a relative axis movement event for all saved relative axis states.
    ///
    /// The saved relative axis state represents a velocity for the relative axis, so this can be
    /// thought of as "integrating" that velocity.
    pub fn pulse_rel_axes(&mut self) {
        let device = &mut self.device;

        self.rel_axes_vals
            .iter()
            .filter(|(_axis, val)| **val != 0)
            .for_each(|(axis, val)| device.rel_axis_move(*axis, *val));
    }

    /// Emits the relevant event and saves the new state for an input [KeyStateChange].
    fn execute_keystate_change(&mut self, change: &KeyStateChange) {
        match change.state {
            PressState::Down => {
                for key in &change.keys {
                    self.down_key(*key);
                }
            }
            PressState::Up => {
                for key in &change.keys {
                    self.up_key(*key);
                }
            }
        }
    }

    /// Saves the new state for an input [AxisList] representing a state change.
    fn execute_axesstate_change(&mut self, change: &AxisList) {
        change.iter().for_each(|rel_axis_info| {
            self.rel_axes_vals
                .insert(rel_axis_info.axis(), rel_axis_info.state());
        });
    }

    /// Executes a [StateChange] output event, emitting events and saving states when applicable.
    fn execute_state_change(&mut self, change: &StateChange) {
        if let Some(keychange) = &change.keys {
            self.execute_keystate_change(keychange);
        }
        if let Some(axeschange) = &change.axes {
            self.execute_axesstate_change(axeschange);
        }
    }

    /// Executes a [Pulse] output event, emitting events and saving states when applicable.
    fn execute_pulse(&mut self, pulse: &Pulse) {
        if let Some(keycodes) = &pulse.keys {
            for key in keycodes {
                self.up_key(*key);
            }
            for key in keycodes {
                self.down_key(*key);
            }
            for key in keycodes {
                self.up_key(*key);
            }
        }
        if let Some(axes) = &pulse.axes {
            axes.iter().for_each(|movement| {
                self.rel_axis_move(movement.axis(), movement.state());
            });
        }
    }

    /// Executes a [Toggle] output event, emitting events and saving states when applicable.
    fn execute_toggle(&mut self, toggle: &Toggle) {
        if let Some(keys) = &toggle.keys {
            for key in keys {
                let curr_state = self.key_states.get(key).unwrap_or(&PressState::Up);
                match curr_state {
                    PressState::Down => {
                        self.up_key(*key);
                    }
                    PressState::Up => {
                        self.down_key(*key);
                    }
                }
            }
        }
        if let Some(axes_change) = &toggle.axes {
            axes_change.iter().for_each(|event| {
                let axis = event.axis();
                let state = event.state();
                let curr_val = self.rel_axes_vals.get(&axis).unwrap_or(&0);
                if curr_val == &state {
                    self.rel_axes_vals.insert(axis, 0);
                } else {
                    self.rel_axes_vals.insert(axis, state);
                }
            });
        }
    }

    /// Executes an [OutputAction], emitting events and saving states when applicable.
    pub fn execute_event(&mut self, event: &OutputAction) {
        match event {
            OutputAction::StateChange(change) => {
                self.execute_state_change(change);
            }
            OutputAction::Pulse(pulse) => {
                self.execute_pulse(pulse);
            }
            OutputAction::Toggle(toggle) => {
                self.execute_toggle(toggle);
            }
        }
    }
}
