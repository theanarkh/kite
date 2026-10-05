use crate::rust::event_loop;
use crate::rust::util;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

const TCP_SERVER_SLOT: i32 = 0;
const TCP_CONNECTION_SLOT: i32 = 0;

const TCP_SERVER_TAG: u16 = 1;
const TCP_CONNECTION_TAG: u16 = 2;

struct TCP {
    listener: Option<TcpListener>,
}

impl TCP {
    fn new(listener: Option<TcpListener>) -> Self {
        Self { listener }
    }

    fn set_listener(&mut self, listener: TcpListener) {
        self.listener = Some(listener);
    }
}

struct Connection {
    socket: TcpStream,
    addr: SocketAddr,
}

impl Connection {
    fn new(socket: TcpStream, addr: SocketAddr) -> Self {
        Self { socket, addr }
    }

    fn get_socket(&self) -> &TcpStream {
        &self.socket
    }

    fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }
}

pub fn read(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 1 {
        util::throw_exception(scope, "buffer is required");
        return;
    }
    if !args.get(0).is_uint8_array() {
        util::throw_exception(scope, "buffer must be Uint8Array");
        return;
    }
    let buffer: v8::Local<v8::Uint8Array> = args.get(0).cast();
    let store = buffer.buffer(scope).unwrap();
    let offset = buffer.byte_offset();
    let len = buffer.byte_length();

    let ptr = store.data().unwrap().as_ptr() as *const u8;

    let conn_ptr =
        unsafe { args.this().get_aligned_pointer_from_internal_field(TCP_CONNECTION_SLOT, TCP_CONNECTION_TAG) as *mut Connection };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move {
            let conn = unsafe { &mut *conn_ptr };
            loop {
                // Wait for the socket to be readable
                let result = conn.get_socket().readable().await;
                if let Err(e) = result {
                    return Err(e.into());
                }
                let buf =
                    unsafe { std::slice::from_raw_parts_mut(ptr.add(offset) as *mut u8, len) };
                match conn.get_socket().try_read(buf) {
                    Ok(0) => return Ok(0),
                    Ok(n) => {
                        return Ok(n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        },
        move |scope: &mut v8::PinScope, result: io::Result<usize>| {
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(n) => {
                    let nread = v8::Number::new(scope, n as f64);
                    resolver.resolve(scope, nread.into());
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn write(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 1 {
        util::throw_exception(scope, "buffer is required");
        return;
    }
    if !args.get(0).is_uint8_array() {
        util::throw_exception(scope, "buffer must be Uint8Array");
        return;
    }
    let buffer: v8::Local<v8::Uint8Array> = args.get(0).cast();
    let store = buffer.buffer(scope).unwrap();
    let offset = buffer.byte_offset();
    let len = buffer.byte_length();

    let ptr = store.data().unwrap().as_ptr() as *const u8;

    let conn_ptr =
        unsafe { args.this().get_aligned_pointer_from_internal_field(TCP_CONNECTION_SLOT, TCP_CONNECTION_TAG) as *mut Connection };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move {
            let conn = unsafe { &mut *conn_ptr };
            loop {
                // Wait for the socket to be readable
                let result = conn.get_socket().writable().await;
                if let Err(e) = result {
                    return Err(e.into());
                }
                let buf =
                    unsafe { std::slice::from_raw_parts_mut(ptr.add(offset) as *mut u8, len) };
                match conn.get_socket().try_write(buf) {
                    Ok(0) => return Ok(0),
                    Ok(n) => {
                        return Ok(n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        },
        move |scope: &mut v8::PinScope, result: io::Result<usize>| {
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(n) => {
                    let nread = v8::Number::new(scope, n as f64);
                    resolver.resolve(scope, nread.into());
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn get_addr(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    let conn_ptr =
        unsafe { args.this().get_aligned_pointer_from_internal_field(TCP_CONNECTION_SLOT, TCP_CONNECTION_TAG) as *mut Connection };
    let conn = unsafe { &mut *conn_ptr };
    let obj = v8::Object::new(scope);
    let port = v8::Number::new(scope, conn.get_addr().port() as f64);
    util::set_property(scope, obj, "port", port.into());
    let ip = v8::String::new(scope, &conn.get_addr().ip().to_string()).unwrap();
    util::set_property(scope, obj, "ip", ip.into());
    return_value.set(obj.into());
}

pub fn accept(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) {
    let tcp_ptr = unsafe { args.this().get_aligned_pointer_from_internal_field(TCP_SERVER_SLOT, TCP_SERVER_TAG) as *mut TCP };
    let tcp = unsafe { &mut *tcp_ptr };
    if let Some(listener) = &tcp.listener {
        let resolver = v8::PromiseResolver::new(scope).unwrap();
        let promise = resolver.get_promise(scope);
        return_value.set(promise.into());
        let resolver = v8::Global::new(scope, resolver);
        let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
        event_loop.submit(
            async move {
                let result = listener.accept().await;
                result
            },
            move |scope: &mut v8::PinScope, result: io::Result<(TcpStream, SocketAddr)>| {
                let resolver = v8::Local::new(scope, &resolver);
                match result {
                    Ok((socket, addr)) => {
                        let connection_template = util::new_default_function_template(scope);
                        util::set_proto_method(scope, connection_template, "read", read);
                        util::set_proto_method(scope, connection_template, "write", write);
                        util::set_proto_method(scope, connection_template, "get_addr", get_addr);
                        let connection_instance = connection_template
                            .instance_template(scope)
                            .new_instance(scope)
                            .unwrap();
                        let connection = Box::new(Connection { socket, addr });
                        let ptr = Box::into_raw(connection) as *mut std::ffi::c_void;
                        connection_instance.set_aligned_pointer_in_internal_field(TCP_CONNECTION_SLOT, ptr, TCP_CONNECTION_TAG);
                        resolver.resolve(scope, connection_instance.into());
                    }
                    Err(e) => {
                        let err = util::new_error(scope, &e.to_string());
                        resolver.reject(scope, err.into());
                    }
                }
            },
        );
    } else {
        let err = util::new_error(scope, "server is not bound");
        scope.throw_exception(err);
    }
}

pub fn bind(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) {
    if args.length() < 1 {
        util::throw_exception(scope, "address must be a string");
        return;
    }

    if !args.get(0).is_string() {
        util::throw_exception(scope, "address must be a string");
        return;
    }

    let address = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let tcp_ptr = unsafe { args.this().get_aligned_pointer_from_internal_field(TCP_SERVER_SLOT, TCP_SERVER_TAG) as *mut TCP };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move { TcpListener::bind(&address).await },
        move |scope: &mut v8::PinScope, result: io::Result<TcpListener>| {
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(listener) => {
                    let undefined = v8::undefined(scope);
                    resolver.resolve(scope, undefined.into());
                    let tcp = unsafe { &mut *tcp_ptr };
                    tcp.set_listener(listener);
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn new(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue<v8::Value>,
) -> () {
    let tcp = Box::new(TCP::new(None));
    let ptr = Box::into_raw(tcp) as *mut std::ffi::c_void;
    args.this().set_aligned_pointer_in_internal_field(TCP_SERVER_SLOT, ptr, TCP_SERVER_TAG);
}

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let net = v8::String::new(scope, "net").unwrap();
    let obj = v8::Object::new(scope);
    let tcp_template = util::new_function_template(scope, "TCP", new, 1);
    util::set_proto_method(scope, tcp_template, "bind", bind);
    util::set_proto_method(scope, tcp_template, "accept", accept);
    let tcp_template_instance = tcp_template.get_function(scope).unwrap();
    util::set_property(scope, obj, "TCP", tcp_template_instance.into());
    core.set(scope, net.into(), obj.into());
}
