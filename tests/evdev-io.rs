use chord2key;
use std::time::Duration;
mod common;

/// This test ensures that the package is imported correctly.
#[test]
fn can_import() {
    assert_eq!(
        chord2key::constants::KeyCode::KEY_ESC,
        chord2key::constants::KeyCode::KEY_ESC
    );
}

/// This test ensures that the test runner can create an OutputDevice. If it fails the first
/// thing that should be checked is user permissions (this test needs to be run with sudo).
#[test]
fn can_create_output_device() -> Result<(), std::io::Error> {
    let _device = chord2key::output::device::OutputDevice::init(Some(
        "chord2key test device: can_create_output_device",
    ))?;
    Ok(())
}

/// This test ensures that we can see the created OutputDevice.
#[test]
fn can_check_output_device() -> Result<(), std::io::Error> {
    let name = "chord2key test device: can_check_output_device";
    let _output_device = chord2key::output::device::OutputDevice::init(Some(&name))?;
    let _input_device = chord2key::input::device::InputDevice::from_name(&name);
    let _input_device = _input_device.ok_or(std::io::Error::from(std::io::ErrorKind::NotFound));
    Ok(())
}

/// This test ensures that we can see the created OutputDevice from the test helper
/// BackgroundInputDevice
#[test]
fn can_create_background_input_device() -> Result<(), std::io::Error> {
    let name = "chord2key test device: can_create_background_input_device";
    let _output_device = chord2key::output::device::OutputDevice::init(Some(&name))?;
    let _input_device = common::BackgroundInputDevice::from_name(&name)?;
    Ok(())
}
/// This test ensures that we can pass OutputActions to the OutputDevice.
#[test]
fn can_run_output_actions() -> Result<(), Box<dyn std::error::Error>> {
    use chord2key::constants::KeyCode;
    use chord2key::output::actions::{OutputAction, Pulse};
    let name = "chord2key test device: can_run_output_actions";
    let output_device = chord2key::output::device::OutputDevice::init(Some(&name))?;
    let output_action =
        OutputAction::Pulse(Pulse::new(Some(vec![KeyCode::BTN_TRIGGER_HAPPY1]), None));
    output_device.execute_event(output_action)?;

    Ok(())
}

/// This test ensures that a single OutputAction creates corresponding events in the OS.
#[test]
fn check_output_actions_single() -> Result<(), Box<dyn std::error::Error>> {
    use chord2key::constants::{KeyCode, PressState};
    use chord2key::events::KeyEvent;
    use chord2key::input::events::InputEvent;
    use chord2key::output::actions::{OutputAction, Pulse};

    //Setup the devices
    let name = "chord2key test device: check_output_actions_single";
    let output_device = chord2key::output::device::OutputDevice::init(Some(&name))?;
    let input_device = common::BackgroundInputDevice::from_name(&name)?;

    // Output the action
    let output_action =
        OutputAction::Pulse(Pulse::new(Some(vec![KeyCode::BTN_TRIGGER_HAPPY1]), None));
    output_device.execute_event(output_action)?;

    std::thread::sleep(Duration::from_millis(common::SLEEP_TIME));
    // Recieve the action
    let events = input_device.collect_timeout(Duration::from_millis(common::TIMEOUT_TIME))?;

    // Check that it was correct
    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY1, PressState::Down));
    assert_eq!(events[0], expected_event);
    
    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY1, PressState::Up));
    assert_eq!(events[1], expected_event);
    Ok(())
}

/// This test ensures that multiple OutputActions create corresponding events in the OS.
#[test]
fn check_output_actions_multiple() -> Result<(), Box<dyn std::error::Error>> {
    use chord2key::constants::{KeyCode, PressState};
    use chord2key::events::KeyEvent;
    use chord2key::input::events::InputEvent;
    use chord2key::output::actions::{OutputAction, Pulse, Toggle, StateChange, KeyStateChange};

    //Setup the devices
    let name = "chord2key test device: check_output_actions_multiple";
    let output_device = chord2key::output::device::OutputDevice::init(Some(&name))?;
    let input_device = common::BackgroundInputDevice::from_name(&name)?;

    // Pulse 1
    let output_action =
        OutputAction::Pulse(Pulse::new(Some(vec![KeyCode::BTN_TRIGGER_HAPPY1]), None));
    output_device.execute_event(output_action)?;

    // Pulse 2 
    let output_action =
        OutputAction::Pulse(Pulse::new(Some(vec![KeyCode::BTN_TRIGGER_HAPPY2]), None));
    output_device.execute_event(output_action)?;

    // Toggle 3
    let output_action =
        OutputAction::Toggle(Toggle::new(Some(vec![KeyCode::BTN_TRIGGER_HAPPY3]), None));
    output_device.execute_event(output_action)?;

    // Toggle 3 again
    let output_action =
        OutputAction::Toggle(Toggle::new(Some(vec![KeyCode::BTN_TRIGGER_HAPPY3]), None));
    output_device.execute_event(output_action)?;

    // Press down 4
    let keys = Some(KeyStateChange {
        keys: vec![KeyCode::BTN_TRIGGER_HAPPY4],
        state: PressState::Down,
    });
    let change = StateChange::new(keys, None);

    let output_action = OutputAction::StateChange(change);
    output_device.execute_event(output_action)?;

    // Press up 4
    let keys = Some(KeyStateChange {
        keys: vec![KeyCode::BTN_TRIGGER_HAPPY4],
        state: PressState::Up,
    });
    let change = StateChange::new(keys, None);

    let output_action = OutputAction::StateChange(change);
    output_device.execute_event(output_action)?;

    // Press up 4 again
    let keys = Some(KeyStateChange {
        keys: vec![KeyCode::BTN_TRIGGER_HAPPY4],
        state: PressState::Up,
    });
    let change = StateChange::new(keys, None);

    let output_action = OutputAction::StateChange(change);
    output_device.execute_event(output_action)?;

    std::thread::sleep(Duration::from_millis(common::SLEEP_TIME));

    // Recieve the actions
    let events = input_device.collect_timeout(Duration::from_millis(common::TIMEOUT_TIME))?;

    // Check that it was correct
    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY1, PressState::Down));
    assert_eq!(events[0], expected_event);
    
    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY1, PressState::Up));
    assert_eq!(events[1], expected_event);

    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY2, PressState::Down));
    assert_eq!(events[2], expected_event);

    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY2, PressState::Up));
    assert_eq!(events[3], expected_event);

    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY3, PressState::Down));
    assert_eq!(events[4], expected_event);

    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY3, PressState::Up));
    assert_eq!(events[5], expected_event);

    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY4, PressState::Down));
    assert_eq!(events[6], expected_event);

    let expected_event =
        InputEvent::KeyEvent(KeyEvent::new(KeyCode::BTN_TRIGGER_HAPPY4, PressState::Up));
    assert_eq!(events[7], expected_event);

    assert_eq!(events.len(), 8);
    Ok(())
}
