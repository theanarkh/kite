use crate::rust::event_loop;
use crate::rust::util;
use hickory_resolver::config::*;
use hickory_resolver::lookup::Ipv4Lookup;
use hickory_resolver::TokioAsyncResolver;

const DNS_RESOLVER_SLOT: i32 = 0;
const DNS_RESOLVER_TAG: u16 = 1;

fn resolve4(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) -> () {
    if args.length() < 1 {
        util::throw_exception(scope, "host is required");
        return;
    }

    let host = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let resolver_ptr =
        unsafe { args.this().get_aligned_pointer_from_internal_field(DNS_RESOLVER_SLOT, DNS_RESOLVER_TAG) as *mut Resolver };

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());

    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move {
            let resolver = unsafe { &mut *resolver_ptr };
            resolver.resolve4(&host).await
        },
        move |scope: &mut v8::PinScope, result: std::io::Result<Ipv4Lookup>| {
            scope
                .get_slot_mut::<event_loop::EventLoop>()
                .unwrap()
                .add_ref(-1);
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok(a_records) => {
                    let result = v8::Array::new(scope, 0);
                    for (i, ip) in a_records.iter().enumerate() {
                        let obj = v8::Object::new(scope);
                        let ip_value = v8::String::new(scope, &ip.to_string()).unwrap().into();
                        util::set_property(scope, obj, "ip", ip_value);
                        let index = v8::Number::new(scope, i as f64).into();
                        result.set(scope, index, obj.into());
                    }
                    resolver.resolve(scope, result.into());
                }
                Err(e) => {
                    let err = util::new_error(scope, &e.to_string());
                    resolver.reject(scope, err.into());
                }
            }
        },
    );
}

struct Resolver {
    resolver: TokioAsyncResolver,
}

impl Resolver {
    fn new() -> Self {
        Self {
            resolver: TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()),
        }
    }

    async fn resolve4(&self, host: &str) -> std::io::Result<Ipv4Lookup> {
        let result = self.resolver.ipv4_lookup(host).await?;
        Ok(result)
    }
}

pub fn new(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue<v8::Value>,
) -> () {
    let resolver = Box::new(Resolver::new());
    let ptr = Box::into_raw(resolver) as *mut std::ffi::c_void;
    args.this().set_aligned_pointer_in_internal_field(DNS_RESOLVER_SLOT, ptr, DNS_RESOLVER_TAG);
}

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let dns = v8::String::new(scope, "dns").unwrap();
    let obj = v8::Object::new(scope);
    let dns_template = util::new_function_template(scope, "Resolver", new, 1);
    util::set_proto_method(scope, dns_template, "resolve4", resolve4);
    let dns_template_instance = dns_template.get_function(scope).unwrap();
    util::set_property(scope, obj, "Resolver", dns_template_instance.into());
    core.set(scope, dns.into(), obj.into());
}
