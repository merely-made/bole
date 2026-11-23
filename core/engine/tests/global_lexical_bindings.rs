#![allow(unused_crate_dependencies)]
//! Cross-script lookup of global lexical bindings compiled before their declarations.

use boa_engine::{Context, JsValue, Script, Source};

#[test]
fn later_script_lexical_bindings_shadow_global_properties() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "globalThis.value = 10;
             function readValue() { return value; }
             function writeValue(next) { value = next; }
             function deleteValue() { return delete value; }
             function readConstant() { return constant; }
             function writeConstant() { constant = 4; }
             function writeClass() { Widget = 5; }",
        ))
        .unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("readValue()")).unwrap(),
        JsValue::from(10)
    );

    context
        .eval(Source::from_bytes(
            "let value = 20; const constant = 3; class Widget {}",
        ))
        .unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("readValue()")).unwrap(),
        JsValue::from(20)
    );
    assert_eq!(
        context.eval(Source::from_bytes("readConstant()")).unwrap(),
        JsValue::from(3)
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "try { writeConstant(); false } catch (error) { error instanceof TypeError }",
            ))
            .unwrap(),
        JsValue::from(true)
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "writeValue(30); writeClass();
                 deleteValue() === false && value === 30 && globalThis.value === 10
                     && Widget === 5 && !Object.hasOwn(globalThis, 'Widget')",
            ))
            .unwrap(),
        JsValue::from(true)
    );
}

#[test]
fn later_script_lexical_bindings_preserve_shadowing_and_tdz() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "globalThis.value = 10;
             function readLocal() { let value = 1; return value; }
             function readWith() { with ({ value: 2 }) { return value; } }
             function readWithFallback() { with ({}) { return value; } }
             function readEvalBinding() { eval('var value = 4'); return value; }
             function readType() { return typeof value; }
             function readMissingType() { return typeof missing; }",
        ))
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "let inTdz;
                 try { readType(); inTdz = false; }
                 catch (error) { inTdz = error instanceof ReferenceError; }
                 let value = 20;
                 inTdz && readLocal() === 1 && readWith() === 2 && readWithFallback() === 20
                     && readEvalBinding() === 4 && readType() === 'number'
                     && readMissingType() === 'undefined'",
            ))
            .unwrap(),
        JsValue::from(true)
    );
}

#[test]
fn parsing_a_later_script_does_not_activate_its_bindings() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "globalThis.value = 10; function readValue() { return value; }",
        ))
        .unwrap();
    let script = Script::parse(Source::from_bytes("let value = 20;"), None, &mut context).unwrap();

    assert_eq!(
        context.eval(Source::from_bytes("readValue()")).unwrap(),
        JsValue::from(10)
    );
    script.evaluate(&mut context).unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("readValue()")).unwrap(),
        JsValue::from(20)
    );
}
