use core::time;
use std::thread::sleep;

use chaotest::init;
use chaotest::INSTRUCTION::*;
fn main() {
    let src = "my url";
    let instruction = OnceDelayWriteMs(2000);

    let ret = init(src, instruction);
    eprintln!("~~ {:?}", ret);

    sleep(time::Duration::from_secs(3600));
}
