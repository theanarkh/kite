use std::os::fd::AsRawFd;
use crate::rust::event_loop;
use crate::rust::util;
use tokio::fs;
use tokio::fs::File;

fn open(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 1 {
        util::throw_exception(scope, "path is required");
        return;
    }

    let path = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());

    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move { File::open(&path).await },
        move |scope: &mut v8::PinScope, result: std::io::Result<File>| {
            scope
                .get_slot_mut::<event_loop::EventLoop>()
                .unwrap()
                .add_ref(-1);
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(file) => {
                    let fd = v8::Number::new(scope, file.as_raw_fd() as f64);
                    resolver.resolve(scope, fd.into());
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

fn read(
    scope: &mut v8::PinScope,
    mut args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 1 {
        let err = util::new_error(scope, "path is required");
        scope.throw_exception(err);
        return;
    }
    
    let path = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());

    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();

    event_loop.submit(
        async move { fs::read_to_string(&path).await },
        move |scope: &mut v8::PinScope, result: std::io::Result<String>| {
            scope
                .get_slot_mut::<event_loop::EventLoop>()
                .unwrap()
                .add_ref(-1);
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(content) => {
                    let s = v8::String::new(scope, &content).unwrap();
                    resolver.resolve(scope, s.into());
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

fn write(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 2 {
        let err = util::new_error(scope, "path and content required");
        scope.throw_exception(err);
        return;
    }

    let path = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let content = args
        .get(1)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());

    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move { fs::write(&path, content.as_bytes()).await },
        move |scope: &mut v8::PinScope, result: std::io::Result<()>| {
            scope
                .get_slot_mut::<event_loop::EventLoop>()
                .unwrap()
                .add_ref(-1);
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(_) => {
                    let s = v8::String::new(scope, "write success").unwrap();
                    resolver.resolve(scope, s.into());
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let fs = v8::String::new(scope, "fs").unwrap();
    let obj = v8::Object::new(scope);
    util::set_method(scope, obj, "open", open);
    util::set_method(scope, obj, "read", read);
    util::set_method(scope, obj, "write", write);
    core.set(scope, fs.into(), obj.into());
}
