use crate::rust::util;

fn write_utf8(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) {
    if args.length() < 1 {
        util::throw_exception(scope, "invalid arguments");
        return;
    }
    if !args.get(0).is_string() {
        util::throw_exception(scope, "argument must be a string");
        return;
    }
    let source = args.get(0).to_string(scope).unwrap();
    let size = source.utf8_length(scope);
    let buffer = v8::ArrayBuffer::new(scope, size);

    let data = unsafe {
        let ptr = buffer.data().unwrap().as_ptr() as *mut u8;
        std::slice::from_raw_parts_mut(ptr, size)
    };
    let _ = source.write_utf8_v2(
        scope,
        data,
        v8::WriteFlags::kReplaceInvalidUtf8,
        None,
    );
    let uint_array = v8::Uint8Array::new(scope, buffer, 0, size).unwrap();
    return_value.set(uint_array.into());
}

fn from_utf8(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) {
    if args.length() < 1 {
        util::throw_exception(scope, "invalid arguments");
        return;
    }
    if !args.get(0).is_uint8_array() {
        util::throw_exception(scope, "argument must be a uint8 array");
        return;
    }
    let uint8_array = args.get(0).cast::<v8::Uint8Array>();
    let store = uint8_array.buffer(scope).unwrap().get_backing_store();
    let data = unsafe {
        let ptr = store.data().unwrap().as_ptr() as *mut u8;
        std::slice::from_raw_parts_mut(ptr, store.byte_length())
    };
    let result = v8::String::new_from_utf8(scope, data, v8::NewStringType::Normal).unwrap();
    return_value.set(result.into());
}

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let buffer = v8::String::new(scope, "buffer").unwrap();
    let obj = v8::Object::new(scope);
    util::set_method(scope, obj, "writeUTF8", write_utf8);
    util::set_method(scope, obj, "fromUTF8", from_utf8);
    core.set(scope, buffer.into(), obj.into());
}
