fn main() {
    use chord2key::input::device::*;
    println!("Please select the device you wish to test\n");
    let mut d = InputDevice::from_cli();
    loop {
        d.poll(|e| {
            println!("{:?}", e);
        })
        .ok();
    }
}
