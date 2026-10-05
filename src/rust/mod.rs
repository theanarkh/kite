pub mod env;
pub mod util;
pub mod event_loop;
pub mod console;
pub mod js;
pub mod loader;
pub mod process;
pub mod buffer;
pub mod fs;
pub mod udp;
pub mod tcp;
pub mod http;
pub mod dns;

pub fn register_modules(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    console::register(scope, core);
    loader::register(scope, core);
    process::register(scope, core);
    buffer::register(scope, core);
    fs::register(scope, core);
    udp::register(scope, core);
    tcp::register(scope, core);
    http::register(scope, core);
    dns::register(scope, core);
}