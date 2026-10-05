use crate::rust::util;

fn log(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue<v8::Value>,
) -> () {
    let msg = args.get(0).to_string(scope).unwrap();
    println!("{}", msg.to_rust_string_lossy(scope));
}

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let console = v8::String::new(scope, "console").unwrap();
    let obj = v8::Object::new(scope);
    util::set_method(scope, obj, "log", log);
    core.set(scope, console.into(), obj.into());
}

