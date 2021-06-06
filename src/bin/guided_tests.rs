#[allow(dead_code)]
mod tests {

    mod input_device {
        use chord2key::input::device::*;
        pub fn test_pick_cli() {
            println!("test_pick_cli()");
            println!("This should allow you to a pick a device and print the first 10 events\n");

            let mut d = InputDevice::from_cli();

            let mut counter = 0;

            while counter < 10 {
                d.poll(|event| {
                    counter += 1;
                    println!("{:?}", event);
                })
                .ok();
            }
            println!("\n");
        }

        pub fn test_pick_name() {
            println!("test_pick_name()");
            println!(
                "This should pick a Nintendo Switch Pro Controller and print its first 10 events\n"
            );

            let d = InputDevice::from_name("Nintendo Switch Pro Controller");
            if d.is_none() {
                panic!("Please connect a Nintendo Switch Pro Controller");
            }
            let mut d = d.unwrap();
            let mut counter = 0;

            while counter < 10 {
                d.poll(|event| {
                    counter += 1;
                    println!("{:?}", event);
                })
                .ok();
            }
            println!("\n");
        }

        pub fn test_all() {
            test_pick_cli();
            test_pick_name();
        }
    }
    mod output_device {
        use chord2key::constants::*;
        use chord2key::output::actions::*;
        use chord2key::output::device::*;
        use std::thread::sleep;
        use std::time::Duration;

        pub fn test_writing() {
            println!("\ntest_writing()");
            println!(
                "This should type out 'hello world' from your keyboard with 30ms per key press after 500ms"
                );
            let board = OutputDevice::init()
                .expect("Could not create test OutputDevice, check the permissions!");

            sleep(Duration::from_millis(500));

            let letters = [
                KeyCode::KEY_H,
                KeyCode::KEY_E,
                KeyCode::KEY_L,
                KeyCode::KEY_L,
                KeyCode::KEY_O,
                KeyCode::KEY_SPACE,
                KeyCode::KEY_W,
                KeyCode::KEY_O,
                KeyCode::KEY_R,
                KeyCode::KEY_L,
                KeyCode::KEY_D,
            ];
            for letter in letters.iter() {
                let pulse: Pulse = Pulse::new(Some(vec![*letter]), None);
                board.execute_event(OutputAction::Pulse(pulse)).ok();
                sleep(Duration::from_millis(30));
            }

            println!("");
        }

        pub fn test_combo_states() {
            println!("\ntest_combo_states()");
            println!("This should type out the letter 'j' and move your mouse left");

            let board = OutputDevice::init()
                .expect("Could not create test OutputDevice, check the permissions!");

            sleep(Duration::from_millis(500));

            let key_change = KeyStateChange {
                keys: vec![KeyCode::KEY_J],
                state: PressState::Down,
            };
            let axis_change = vec![(RelAxisCode::REL_X, -5)].into();

            let mut change = StateChange {
                keys: Some(key_change),
                axes: Some(axis_change),
            };
            board
                .execute_event(OutputAction::StateChange(change.clone()))
                .ok();

            sleep(Duration::from_millis(1000));
            change.inverse();
            board.execute_event(OutputAction::StateChange(change)).ok();
            sleep(Duration::from_millis(1000));

            println!("");
        }

        pub fn test_all() {
            test_writing();
            test_combo_states();
        }
    }
    pub fn test_all() {
        input_device::test_all();
        //output_device::test_all();
    }
}
fn main() {
    tests::test_all();
}
