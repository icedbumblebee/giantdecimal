#![no_main]
use decimal::Decimal;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|pair: (Decimal, Decimal)| {
    let (a, b) = pair;
    // Basic algebraic laws that must hold
    dbg!(&a);
    dbg!(&b);
    // assert_eq!(a.clone() + b.clone(), b + a.clone()); // commutativity
    // let zero = a.clone() - a.clone();
    // assert!(zero.is_zero(), "a - a should be zero; got {:?}", zero);
    assert_eq!((a.clone() * Decimal::try_from("1").unwrap()), a); // identity
});
