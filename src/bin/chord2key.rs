use chord2key::input::device::*;
use chord2key::mapping::mapper::*;
use chord2key::output::device::*;
use std::env;

fn print_usage_and_exit() {
    eprintln!("Usage: chord2key [PATH_TO_CONFIG_FILE]");
    std::process::exit(1);
}
fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() > 2 {
        eprintln!("Too many arguments!");
        print_usage_and_exit();
    }
    if args.len() < 2 {
        eprintln!("No configuration file specified!");
        print_usage_and_exit();
    }
    let config_path = &args[1];

    println!("Starting keyboard + mouse emulation...");
    let output_device = OutputDevice::init().expect("Could not initialize the Output Device");
    println!("Started keyboard + mouse emulation!\n");

    /*
    let config = chord2key::mapping::configuration::Configuration::r_joycon_mouse();
    config.save_to_file(config_path).ok();
    */
    /*
    let config = chord2key::mapping::configuration::Configuration::joycon_default();
    config.save_to_file(config_path).ok();
    */

    println!("Loading configuration file(s)...");
    let mut mapper = Mapper::init_from_file(output_device, config_path).unwrap();
    println!("Configuration file(s) successfully loaded!\n");

    println!("Searching for {}...", mapper.get_input_name());
    let mut input_device = match InputDevice::from_name(mapper.get_input_name()) {
        Some(device) => device,
        None => {
            eprintln!(
                "Could not find required input device: {}",
                mapper.get_input_name()
            );
            std::process::exit(1);
        }
    };
    println!("Successfully found {}!\n", mapper.get_input_name());

    println!(
        "chord2key will now convert {} events into keyboard+mouse events!",
        mapper.get_input_name()
    );

    loop {
        let result = input_device.poll(|ev| {
            mapper.handle_event(ev);
        });
        if let Err(e) = result {
            match e.kind() {
                std::io::ErrorKind::Other => {
                    if let Some(raw_os_err) = e.raw_os_error() {
                        if raw_os_err == 19 {
                            eprintln!("The device got disconnected!");
                            std::process::exit(1);
                        }
                    }
                }
                _ => {
                    eprintln!("An unknown error occured: ");
                    eprintln!("{:?}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
