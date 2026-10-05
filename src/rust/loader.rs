use crate::rust::js;
use crate::rust::util;

fn compile(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 1 || !args.get(0).is_string() {
        util::throw_exception(scope, "path is invalid");
        return;
    }
    let path = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let result = std::fs::read_to_string(&path);
    if let Err(e) = result {
        util::throw_exception(scope, &e.to_string());
        return;
    }
    let code = v8::String::new(scope, &result.unwrap()).unwrap();
    let mut source = v8::script_compiler::Source::new(code, None);
    let arg_keys: &[v8::Local<'_, v8::String>] = &[
        v8::String::new(scope, "require").unwrap(),
        v8::String::new(scope, "exports").unwrap(),
        v8::String::new(scope, "module").unwrap(),
        v8::String::new(scope, "__filename").unwrap(),
        v8::String::new(scope, "__dirname").unwrap(),
    ];
    let compile_result = v8::script_compiler::compile_function(
        scope,
        &mut source,
        &arg_keys,
        &[],
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::NoReason,
    );
    if let Some(func) = compile_result {
        return_value.set(func.into());
    } else {
        util::throw_exception(scope, "compile failed");
    }
}

fn internal_compile(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 1 || !args.get(0).is_string() {
        util::throw_exception(scope, "path is invalid");
        return;
    }
    let path = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let js_code = js::get_js_code(path.as_str());
    if js_code.is_none() {
        util::throw_exception(scope, "path not found"); 
        return;
    }
    let code = v8::String::new(scope, &js_code.unwrap()).unwrap();
    let mut source = v8::script_compiler::Source::new(code, None);
    let arg_keys: &[v8::Local<'_, v8::String>] = &[
        v8::String::new(scope, "require").unwrap(),
        v8::String::new(scope, "exports").unwrap(),
        v8::String::new(scope, "module").unwrap(),
        v8::String::new(scope, "__filename").unwrap(),
        v8::String::new(scope, "__dirname").unwrap(),
        v8::String::new(scope, "core").unwrap(),
    ];
    let compile_result = v8::script_compiler::compile_function(
        scope,
        &mut source,
        &arg_keys,
        &[],
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::NoReason,
    );
    if let Some(func) = compile_result {
        return_value.set(func.into());
    } else {
        util::throw_exception(scope, "compile failed");
    }
}

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let loader = v8::String::new(scope, "loader").unwrap();
    let obj = v8::Object::new(scope);
    util::set_method(scope, obj, "compile", compile);
    util::set_method(scope, obj, "internalCompile", internal_compile);
    core.set(scope, loader.into(), obj.into());
}
