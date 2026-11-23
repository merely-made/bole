#![allow(unused_crate_dependencies)]
//! Lexical `this` across chains of arrow functions.

use boa_engine::{Context, JsValue, Source};

#[test]
fn nested_arrows_capture_the_method_receiver() {
    let mut context = Context::default();
    let value = context
        .eval(Source::from_bytes(
            "const object = { make() { return () => () => this; } };
             const inner = object.make().call({});
             inner.call({}) === object",
        ))
        .unwrap();
    assert_eq!(value, JsValue::from(true));
}

#[test]
fn nested_arrows_capture_the_nearest_ordinary_function() {
    let mut context = Context::default();
    let value = context
        .eval(Source::from_bytes(
            "function outer() {
                 return function make() {
                     return (callback = () => this) => callback;
                 };
             }
             const receiver = {};
             const make = outer.call({});
             const inner = make.call(receiver)();
             inner() === receiver",
        ))
        .unwrap();
    assert_eq!(value, JsValue::from(true));
}

#[test]
fn direct_and_single_arrow_this_keep_their_owners() {
    let mut context = Context::default();
    let value = context
        .eval(Source::from_bytes(
            "const object = { direct() { return this; }, make() { return () => this; } };
             object.direct() === object && object.make()() === object
                 && (() => () => this)()() === globalThis",
        ))
        .unwrap();
    assert_eq!(value, JsValue::from(true));
}
