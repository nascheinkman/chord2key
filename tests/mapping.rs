use chord2key;
use std::env;
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

/// This test ensures that the config files can be created properly
#[test]
#[ignore]
fn can_create_config() -> Result<(), Box<dyn std::error::Error>> {
    common::config_generator::generate_blank("/generate_blank/generate_blank.json")?;
    common::config_generator::generate_one_to_one("/one_to_one/one_to_one.json")?;
    common::config_generator::generate_multiple_to_one("/multiple_to_one/multiple_to_one.json")?;
    Ok(())
}

/// This test ensures that the config files can be loaded properly
#[test]
fn can_load_config() -> Result<(), Box<dyn std::error::Error>> {
    let filename = common::get_test_config_dir()? + "/generate_blank/generate_blank.json";
    chord2key::mapping::configuration::Configuration::load_from_file(filename)?;
    Ok(())
}

/// This test ensures that the config files can be read properly
#[test]
fn can_read_config() -> Result<(), Box<dyn std::error::Error>> {
    let filename = common::get_test_config_dir()? + "/one_to_one/one_to_one.json";
    let config = chord2key::mapping::configuration::Configuration::load_from_file(filename)?;
    let _input1: chord2key::mapping::mapper::ChordInput =
        chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY1.into();

    let _output1: chord2key::mapping::actions::Action = chord2key::output::actions::Pulse::new(
        Some(vec![chord2key::constants::KeyCode::BTN_TRIGGER_HAPPY2]),
        None,
    )
    .into();

    assert_eq!(config.chord_inputs.len(), 1);
    assert!(matches!(config.chord_inputs[0], _input1));
    assert_eq!(config.chord_mapping.len(), 1);
    assert!(matches!(&config.chord_mapping[0], _output1));

    Ok(())
}

/// This test ensures that the config file can be successfully used in a mapper object
#[test]
fn can_map_config() -> Result<(), Box<dyn std::error::Error>> {
    let device_name = "chord2key test device: can_map_config";
    let filename = common::get_test_config_dir()? + "/one_to_one/one_to_one.json";
    let output_device = chord2key::output::device::OutputDevice::init(Some(&device_name))?;
    let _mapper = chord2key::mapping::mapper::Mapper::init_from_file(output_device, filename)?;
    Ok(())
}

/// This test ensures that single keypress events are successfully mapped
#[test]
fn can_map_single_keypress_events() -> Result<(), Box<dyn std::error::Error>> {
    use chord2key::constants::{KeyCode, PressState};
    use chord2key::input::events::InputEvent;

    let mapping_output_name = "chord2key test device: can_map_single_keypress_events";
    let filename = common::get_test_config_dir()? + "/one_to_one/one_to_one.json";
    let background_mapper =
        common::BackgroundMapper::create_with_config(&filename, mapping_output_name)?;

    background_mapper.keypulse_to_input(KeyCode::BTN_TRIGGER_HAPPY1)?;

    std::thread::sleep(Duration::from_millis(common::SLEEP_TIME));
    // Recieve the action
    let events = background_mapper
        .output_reader
        .collect_timeout(Duration::from_millis(common::TIMEOUT_TIME))?;

    // Check that it was correct
    assert_eq!(events.len(), 2);
    let expected_event: InputEvent = (KeyCode::BTN_TRIGGER_HAPPY2, PressState::Down).into();
    assert_eq!(events[0], expected_event);

    let expected_event: InputEvent = (KeyCode::BTN_TRIGGER_HAPPY2, PressState::Up).into();
    assert_eq!(events[1], expected_event);

    Ok(())
}

/// This test ensures that multiple keypress events are successfully mapped
#[test]
fn can_map_multiple_keypress_events() -> Result<(), Box<dyn std::error::Error>> {
    use chord2key::constants::{KeyCode, PressState};
    use chord2key::input::events::InputEvent;

    let mapping_output_name = "chord2key test device: can_map_multiple_keypress_events";
    let filename = common::get_test_config_dir()? + "/multiple_to_one/multiple_to_one.json";
    let background_mapper =
        common::BackgroundMapper::create_with_config(&filename, mapping_output_name)?;

    background_mapper.keychange_to_input(KeyCode::BTN_TRIGGER_HAPPY1, PressState::Down)?;
    background_mapper.keychange_to_input(KeyCode::BTN_TRIGGER_HAPPY2, PressState::Down)?;
    background_mapper.keychange_to_input(KeyCode::BTN_TRIGGER_HAPPY3, PressState::Down)?;
    background_mapper.keychange_to_input(KeyCode::BTN_TRIGGER_HAPPY2, PressState::Up)?;

    std::thread::sleep(Duration::from_millis(common::SLEEP_TIME));
    // Recieve the action
    let events = background_mapper
        .output_reader
        .collect_timeout(Duration::from_millis(common::TIMEOUT_TIME))?;

    // Check that it was correct
    assert_eq!(events.len(), 2);

    let expected_event: InputEvent = (KeyCode::BTN_TRIGGER_HAPPY4, PressState::Down).into();
    assert_eq!(events[0], expected_event);

    let expected_event: InputEvent = (KeyCode::BTN_TRIGGER_HAPPY4, PressState::Up).into();
    assert_eq!(events[1], expected_event);

    Ok(())
}
