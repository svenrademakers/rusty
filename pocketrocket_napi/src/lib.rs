#[macro_use]
extern crate napi_derive;

use napi::*;
use napi::{JsObject, Result};

use std::convert::TryInto;

#[js_function(1)] // ------> arguments length, omit for zero
fn fibonacci(ctx: CallContext) -> Result<JsNumber> {
    let n = ctx.get::<JsNumber>(0)?.try_into()?;
    ctx.env.create_int64(fibonacci_native(n))
}

#[inline(always)]
fn fibonacci_native(n: i64) -> i64 {
    match n {
        1 | 2 => 1,
        _ => fibonacci_native(n - 1) + fibonacci_native(n - 2),
    }
}

/// `exports` is `module.exports` object in NodeJS
#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("fibonacci", fibonacci)?;
    Ok(())
}
