#![no_main]
use libfuzzer_sys::fuzz_target;

use exmex;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data){
        let _ = exmex::eval_str::<f64>(s);
    }
});
