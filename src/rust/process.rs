use crate::rust::util;
use crate::rust::env;

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let process = v8::String::new(scope, "process").unwrap();
    let obj = v8::Object::new(scope);
    let argv: v8::Local<'_, v8::Array> = v8::Array::new(scope, scope.get_slot::<env::Env>().unwrap().argv().len() as i32);
    for (i, arg) in scope.get_slot::<env::Env>().unwrap().argv().iter().enumerate() {
        let index = v8::Number::new(scope, i as f64);
        let value = v8::String::new(scope, arg).unwrap();
        argv.set(scope, index.into(), value.into());
    }
    util::set_property(scope, obj, "argv", argv.into());
    let is_main_thread = v8::Boolean::new(scope, scope.get_slot::<env::Env>().unwrap().is_main_thread());
    util::set_property(scope, obj, "isMainThread", is_main_thread.into());
    core.set(scope, process.into(), obj.into());
}
