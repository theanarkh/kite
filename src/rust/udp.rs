use crate::rust::event_loop;
use crate::rust::util;
use std::io;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

const UDP_SOCKET_SLOT: i32 = 0;
const UDP_SOCKET_TAG: u16 = 1;

struct UDP {
    listener: Option<UdpSocket>,
    connected: bool,
    // 指向 JS 包装对象的弱引用：不阻止对象被 GC；对象回收（或 isolate 拆除）
    // 时通过 guaranteed finalizer 执行 Box::from_raw 回收本结构。
    weak: Option<v8::Weak<v8::Object>>,
}

impl UDP {
    fn new(listener: Option<UdpSocket>) -> Self {
        Self {
            listener,
            connected: false,
            weak: None,
        }
    }

    fn set_listener(&mut self, listener: UdpSocket) {
        self.listener = Some(listener);
    }

    fn get_listener(&self) -> &UdpSocket {
        self.listener.as_ref().unwrap()
    }

    async fn connect(&mut self, address: &String) -> io::Result<()> {
        let result = self.get_listener().connect(&address).await;
        if let Ok(_) = result {
            self.connected = true;
        }
        result
    }

    async fn recv(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let result = self.listener.as_mut().unwrap().readable().await;
        if let Err(e) = result {
            return Err(e.into());
        }
        self.listener.as_mut().unwrap().recv_from(buf).await
    }

    async fn send(&mut self, buf: &[u8], address: Option<SocketAddr>) -> io::Result<usize> {
        if self.listener.is_none() {
            let bind_addr = match &address {
                Some(SocketAddr::V4(_)) => "0.0.0.0:0",
                Some(SocketAddr::V6(_)) => "[::]:0",
                None => "0.0.0.0:0",
            };
            let socket = UdpSocket::bind(bind_addr).await?;
            self.listener = Some(socket);
        }
        if self.connected {
            self.listener.as_mut().unwrap().send(buf).await
        } else {
            self.listener
                .as_mut()
                .unwrap()
                .send_to(buf, &address.unwrap())
                .await
        }
    }
}

pub fn recv(
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

    let udp_ptr = unsafe { args.this().get_aligned_pointer_from_internal_field(UDP_SOCKET_SLOT, UDP_SOCKET_TAG) as *mut UDP };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move {
            let udp = unsafe { &mut *udp_ptr };
            loop {
                let mut buf =
                    unsafe { std::slice::from_raw_parts_mut(ptr.add(offset) as *mut u8, len) };
                let result = udp.recv(&mut buf).await;
                match result {
                    Ok((n, addr)) => {
                        return Ok((n as usize, addr));
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
        move |scope: &mut v8::PinScope, result: io::Result<(usize, SocketAddr)>| {
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok((n, addr)) => {
                    let nread = v8::Number::new(scope, n as f64);
                    let addr = v8::String::new(scope, &addr.to_string()).unwrap();
                    let obj = v8::Object::new(scope);
                    util::set_property(scope, obj, "nread", nread.into());
                    util::set_property(scope, obj, "addr", addr.into());
                    resolver.resolve(scope, obj.into());
                }
                Err(e) => {
                    let err = v8::String::new(scope, &e.to_string()).unwrap();
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn send(
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
    let mut address: Option<SocketAddr> = None;
    if args.length() > 1 {
        if !args.get(1).is_string() {
            util::throw_exception(scope, "address must be String");
            return;
        }
        let address_str = util::get_string(scope, args.get(1));
        address = Some(address_str.parse().unwrap());
    }

    let buffer: v8::Local<v8::Uint8Array> = args.get(0).cast();
    let store = buffer.buffer(scope).unwrap();
    let offset = buffer.byte_offset();
    let len = buffer.byte_length();

    let ptr = store.data().unwrap().as_ptr() as *const u8;

    let udp_ptr = unsafe { args.this().get_aligned_pointer_from_internal_field(UDP_SOCKET_SLOT, UDP_SOCKET_TAG) as *mut UDP };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move {
            let udp = unsafe { &mut *udp_ptr };
            loop {
                let result = udp.listener.as_mut().unwrap().writable().await;
                if let Err(e) = result {
                    return Err(e.into());
                }
                let buf = unsafe { std::slice::from_raw_parts(ptr.add(offset) as *const u8, len) };
                let result = udp.send(buf, address).await;
                match result {
                    Ok(n) => {
                        return Ok(n);
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
                    let err = v8::String::new(scope, &e.to_string()).unwrap();
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn bind(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) {
    if args.length() < 1 {
        util::throw_exception(scope, "address is required");
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

    let udp_ptr = unsafe { args.this().get_aligned_pointer_from_internal_field(UDP_SOCKET_SLOT, UDP_SOCKET_TAG) as *mut UDP };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move { UdpSocket::bind(&address).await },
        move |scope: &mut v8::PinScope, result: io::Result<UdpSocket>| {
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(listener) => {
                    let undefined = v8::undefined(scope);
                    resolver.resolve(scope, undefined.into());
                    let udp = unsafe { &mut *udp_ptr };
                    udp.set_listener(listener);
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn connect(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) {
    if args.length() < 1 {
        util::throw_exception(scope, "address is required");
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

    let udp_ptr = unsafe { args.this().get_aligned_pointer_from_internal_field(UDP_SOCKET_SLOT, UDP_SOCKET_TAG) as *mut UDP };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move {
            let udp = unsafe { &mut *udp_ptr };
            udp.connect(&address).await
        },
        move |scope: &mut v8::PinScope, result: io::Result<()>| {
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(()) => {
                    let undefined = v8::undefined(scope);
                    resolver.resolve(scope, undefined.into());
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

pub fn close(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    let udp_ptr = unsafe {
        args.this()
            .get_aligned_pointer_from_internal_field(UDP_SOCKET_SLOT, UDP_SOCKET_TAG) as *mut UDP
    };

    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.add_ref(-1);

    // 只关闭 fd：take() 取出并 drop UdpSocket（close(2) + 注销 mio），
    // UDP 结构体本身不释放，等 JS 对象被 GC 时由 weak finalizer 回收。
    if !udp_ptr.is_null() {
        let udp = unsafe { &mut *udp_ptr };
        udp.listener.take();
        udp.connected = false;
    }

    // 关闭是同步操作，无需提交异步任务，直接 resolve。
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let undefined = v8::undefined(scope);
    resolver.resolve(scope, undefined.into());
}


pub fn new(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue<v8::Value>,
) -> () {
    let udp = Box::new(UDP::new(None));
    let udp_ptr = Box::into_raw(udp);
    let this = args.this();
    this.set_aligned_pointer_in_internal_field(
        UDP_SOCKET_SLOT,
        udp_ptr as *mut std::ffi::c_void,
        UDP_SOCKET_TAG,
    );

    // 弱引用不阻止 JS 对象被 GC。guaranteed finalizer 在对象被回收时
    // （或 isolate 销毁前）必然执行一次：Box::from_raw 回收 UDP 结构；
    // 若用户没调 close，drop Option<UdpSocket> 也会顺带关闭 fd。
    let weak = v8::Weak::with_guaranteed_finalizer(
        &mut **scope,
        this,
        Box::new(move || unsafe {
            // println!("UDP finalizer");
            drop(Box::from_raw(udp_ptr));
        }),
    );
    unsafe { &mut *udp_ptr }.weak = Some(weak);
}

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let udp = v8::String::new(scope, "udp").unwrap();
    let obj = v8::Object::new(scope);
    let udp_template = util::new_function_template(scope, "UDP", new, 1);
    util::set_proto_method(scope, udp_template, "bind", bind);
    util::set_proto_method(scope, udp_template, "connect", connect);
    util::set_proto_method(scope, udp_template, "recv", recv);
    util::set_proto_method(scope, udp_template, "send", send);
    util::set_proto_method(scope, udp_template, "close", close);
    let udp_template_instance = udp_template.get_function(scope).unwrap();
    util::set_property(scope, obj, "UDP", udp_template_instance.into());
    core.set(scope, udp.into(), obj.into());
}
