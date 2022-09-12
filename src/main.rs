use core::time;
use std::thread::sleep;

use chaotest::init;
use chaotest::INSTRUCTION::OnceDelayWriteMs;
fn main() {
    let src = "my url";
    let instruction = OnceDelayWriteMs(1000);

    let ret = init(src, instruction);
    eprintln!("~~ {:?}", ret);

    sleep(time::Duration::from_secs(3600));
}
