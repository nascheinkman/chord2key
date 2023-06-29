use chord2key::input::{device::InputDevice, events::InputEvent};
use chord2key::mapping::configuration::Configuration;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

pub const SLEEP_TIME: u64 = 500;
pub const TIMEOUT_TIME: u64 = 1000;

/// Helper struct for reading input from a background thread. Accumulates recieved input for later
/// use.
pub struct BackgroundInputDevice {
    event_rx: Receiver<InputEvent>,
}

impl BackgroundInputDevice {
    pub fn from_name(name: &str) -> Result<Self, std::io::Error> {
        let mut input_device = InputDevice::from_name(name).ok_or(std::io::ErrorKind::NotFound)?;
        let (tx, rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

        let _handle = thread::spawn(move || loop {
            let mut failed = false;
            let res = input_device.poll(|ev| {
                let event_res = tx.send(ev.clone());
                if event_res.is_err() {
                    failed = true;
                }
            });
            if failed || res.is_err() {
                break;
            }
        });
        Ok(Self { event_rx: rx })
    }

    pub fn collect_timeout(&self, timeout: Duration) -> Result<Vec<InputEvent>, RecvTimeoutError> {
        let mut returned = vec![];
        let recv_event = self.event_rx.recv_timeout(timeout)?;
        let mut recv_event = Ok(recv_event);
        while recv_event.is_ok() {
            returned.push(recv_event.unwrap());
            recv_event = self.event_rx.try_recv();
        }
        Ok(returned)
    }
}

pub struct BackgroundMapper {
    pub input_sender: chord2key::output::device::OutputDevice,
    pub output_reader: crate::common::BackgroundInputDevice,
}

impl BackgroundMapper {
    pub fn create_with_config(
        config_path: &str,
        output_device_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Configuration::load_from_file(config_path)?;
        let config_path = String::from(config_path);
        let mapping_input_name = config.device_name;
        let map_input_device =
            chord2key::output::device::OutputDevice::init(Some(&mapping_input_name))?;
        let mut map_input_reader =
            chord2key::input::device::InputDevice::from_name(&mapping_input_name)
                .ok_or("Could not read from the input device")?;
        let map_output_device =
            chord2key::output::device::OutputDevice::init(Some(&output_device_name))?;

        let map_output_reader =
            crate::common::BackgroundInputDevice::from_name(&output_device_name)?;

        let _handle = thread::spawn(move || {
            let mut mapper =
                chord2key::mapping::mapper::Mapper::init_from_file(map_output_device, config_path)
                    .unwrap();
            loop {
                let _ = map_input_reader.poll(|ev| mapper.handle_event(ev));
            }
        });

        Ok(Self {
            input_sender: map_input_device,
            output_reader: map_output_reader,
        })
    }

    pub fn keypulse_to_input(
        &self, 
        key: chord2key::constants::KeyCode
    ) -> Result<(), Box<dyn std::error::Error>> {
        use chord2key::output::actions::{OutputAction, Pulse};

        let output_action =
            OutputAction::Pulse(Pulse::new(Some(vec![key]), None));
        self.input_sender
            .execute_event(output_action)?;

        Ok(())
    }

    pub fn keychange_to_input(
        &self, 
        key: chord2key::constants::KeyCode,
        statechange: chord2key::constants::PressState
    ) -> Result<(), Box<dyn std::error::Error>> {
        use chord2key::output::actions::{OutputAction, StateChange, KeyStateChange};

        let keys = Some(KeyStateChange {
            keys: vec![key],
            state: statechange,
        });
        let change = StateChange::new(keys, None);
    
        let output_action = OutputAction::StateChange(change);
        self.input_sender
            .execute_event(output_action)?;

        Ok(())
    }

}

pub fn get_test_config_dir() -> Result<String, Box<dyn std::error::Error>> {
    let code_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    Ok(code_dir + "/configs/tests")
}

pub mod config_generator {
    use crate::common::get_test_config_dir;
    
    /// Generates an empty Configuration
    pub fn generate_blank(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filename = get_test_config_dir()? + name;
        let device_name = String::from(name);

        let blank_config = chord2key::mapping::configuration::Configuration {
            device_name,
            axis_thresholds: vec![],
            chord_inputs: vec![],
            chord_mapping: vec![],
            modifier_mapping: vec![],
            mouse_mapping: vec![],
        };
        blank_config.save_to_file(filename)?;

        Ok(())
    }

    /// Generates a Configuration that maps one KeyCode to one KeyCode
    pub fn generate_one_to_one(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filename = get_test_config_dir()? + name;
        let device_name = String::from("chord2key test device: ") + name;

        let input1: chord2key::mapping::mapper::ChordInput =
            chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY1.into();

        let output1: chord2key::mapping::actions::Action = chord2key::output::actions::Pulse::new(
            Some(vec![chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY2]),
            None,
        )
        .into();

        let one_to_one_config = chord2key::mapping::configuration::Configuration {
            device_name,
            axis_thresholds: vec![],
            chord_inputs: vec![input1],
            chord_mapping: vec![(vec![input1], output1)],
            modifier_mapping: vec![],
            mouse_mapping: vec![],
        };
        one_to_one_config.save_to_file(filename)?;

        Ok(())
    }

    /// Generates a Configuration that maps multiple KeyCodes to one KeyCode
    pub fn generate_multiple_to_one(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filename = get_test_config_dir()? + name;
        let device_name = String::from("chord2key test device: ") + name;

        let input1: chord2key::mapping::mapper::ChordInput =
            chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY1.into();

        let input2: chord2key::mapping::mapper::ChordInput =
            chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY2.into();

        let input3: chord2key::mapping::mapper::ChordInput =
            chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY3.into();


        let output1: chord2key::mapping::actions::Action = chord2key::output::actions::Pulse::new(
            Some(vec![chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY4]),
            None,
        )
        .into();

        let multiple_to_one_config = chord2key::mapping::configuration::Configuration {
            device_name,
            axis_thresholds: vec![],
            chord_inputs: vec![input1, input2, input3],
            chord_mapping: vec![(vec![input1, input2, input3], output1)],
            modifier_mapping: vec![],
            mouse_mapping: vec![],
        };
        multiple_to_one_config.save_to_file(filename)?;

        Ok(())
    }

}
