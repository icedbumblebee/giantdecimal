#![no_main]
use decimal::Decimal;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|d: Decimal| {
    let txt = d.to_string();
    let d2 = Decimal::try_from(&txt).expect("parse failed");
});
