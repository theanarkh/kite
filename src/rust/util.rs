
pub fn throw_exception(scope: &mut v8::PinScope, str: &str) {
    let str = v8::String::new(scope, str).unwrap();
    let err = v8::Exception::error(scope, str).into();
    scope.throw_exception(err);
}

pub fn new_error<'a>(scope: &mut v8::PinScope<'a, '_, v8::Context>, str: &str) -> v8::Local<'a, v8::Value> {
    let str = v8::String::new(scope, str).unwrap();
    v8::Exception::error(scope, str).into()
}

pub fn get_string(scope: &mut v8::PinScope, args: v8::Local<'_, v8::Value>) -> String {
    args.cast::<v8::String>()
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope)
}

pub fn set_method(
    scope: &mut v8::PinScope,
    obj: v8::Local<'_, v8::Object>,
    name: &str,
    func: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let template = v8::FunctionTemplate::new(scope, func);
    let name = v8::String::new(scope, name).unwrap().into();
    let func = template.get_function(scope).unwrap();
    func.set_name(name);
    obj.set(scope, name.into(), func.into());
}

fn null_template_callback(
    _: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue<v8::Value>,
) {
    args.this()
        .set_aligned_pointer_in_internal_field(0, std::ptr::null(), 0);
}

pub fn new_default_function_template<'a>(
    scope: &mut v8::PinScope<'a, '_, v8::Context>,
) -> v8::Local<'a, v8::FunctionTemplate> {
    let template = new_function_template(scope, "default", null_template_callback, 1);
    template
}

pub fn new_function_template<'a>(
    scope: &mut v8::PinScope<'a, '_, v8::Context>,
    name: &str,
    func: impl v8::MapFnTo<v8::FunctionCallback>,
    internal_field_count: usize,
) -> v8::Local<'a, v8::FunctionTemplate> {
    let template = v8::FunctionTemplate::new(scope, func);
    template
        .instance_template(scope)
        .set_internal_field_count(internal_field_count);
    let name = v8::String::new(scope, name).unwrap().into();
    template.set_class_name(name);
    template
}

pub fn set_property(
    scope: &mut v8::PinScope<'_, '_, v8::Context>,
    obj: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    let name = v8::String::new(scope, name).unwrap().into();
    obj.set(scope, name, value);
}

pub fn set_proto_method(
    scope: &mut v8::PinScope<'_, '_, v8::Context>,
    template: v8::Local<'_, v8::FunctionTemplate>,
    name: &str,
    func: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let func_name = v8::String::new(scope, name).unwrap();
    let func = new_function_template(scope, "default", func, 0);
    template
        .prototype_template(scope)
        .set(func_name.into(), func.into());
}


