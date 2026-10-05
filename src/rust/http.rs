use crate::rust::event_loop;
use crate::rust::util;
use reqwest;

pub fn request(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut return_value: v8::ReturnValue<v8::Value>,
) {
    if args.length() < 2 {
        let err = util::new_error(scope, "invalid arguments");
        scope.throw_exception(err);
        return;
    }
    let url = util::get_string(scope, args.get(0));
    let options = args.get(1).cast::<v8::Object>();
    let method_key = v8::String::new(scope, "method").unwrap().into();
    let method = options
        .get(scope, method_key)
        .unwrap()
        .cast::<v8::String>()
        .to_string(scope)
        .unwrap();
    let method = util::get_string(scope, method.into());
    let mut body: Option<String> = None;
    if method == "POST" {
        let data_key = v8::String::new(scope, "data").unwrap().into();
        let data = options.get(scope, data_key).unwrap();
        let data = v8::json::stringify(scope, data).unwrap();
        body = Some(util::get_string(scope, data.cast::<v8::Value>()));
    }
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    return_value.set(promise.into());
    let resolver = v8::Global::new(scope, resolver);
    let event_loop = scope.get_slot_mut::<event_loop::EventLoop>().unwrap();
    event_loop.submit(
        async move {
            if method == "GET" {
                let result = reqwest::get(url).await;
                match result {
                    Ok(resp) => {
                        let status = resp.status();
                        let headers = resp.headers().clone();
                        let body = resp.text().await;
                        match body {
                            Ok(body) => Ok((status, headers, body)),
                            Err(e) => Err(e.to_string().clone()),
                        }
                    }
                    Err(e) => Err(e.to_string().clone()),
                }
            } else if method == "POST" {
                let client = reqwest::Client::new();
                let result = client.post(url).body(body.unwrap()).send().await;
                match result {
                    Ok(resp) => {
                        let status = resp.status();
                        let headers = resp.headers().clone();
                        let body = resp.text().await;
                        match body {
                            Ok(body) => Ok((status, headers, body)),
                            Err(e) => Err(e.to_string().clone()),
                        }
                    }
                    Err(e) => Err(e.to_string().clone()),
                }
            } else {
                Err("not implemented".to_string())
            }
        },
        move |scope: &mut v8::PinScope,
              result: Result<(reqwest::StatusCode, reqwest::header::HeaderMap, String), String>| {
            let resolver = v8::Local::new(scope, &resolver);
            match result {
                Ok((status, headers, body)) => {
                    let result = v8::Object::new(scope);
                    let status = v8::Number::new(scope, status.as_u16() as f64).into();
                    util::set_property(
                        scope,
                        result,
                        "status",
                        status,
                    );
                    let header = v8::Object::new(scope);
                    for (key, value) in headers.iter() {
                        let k: v8::Local<'_, v8::Value> = v8::String::new(scope, key.as_str()).unwrap().into();
                        let v = v8::String::new(scope, value.to_str().unwrap()).unwrap().into();
                        let values = header.get(scope, k);
                        if values.unwrap().is_undefined() {
                            let arr = v8::Array::new(scope, 0);
                            arr.set_index(scope, arr.length(), v);
                            util::set_property(
                                scope,
                                header,
                                key.as_str(),
                                arr.into(),
                            ); 
                        } else {
                            let arr = values.unwrap().cast::<v8::Array>();
                            arr.set_index(scope, arr.length(), v);
                        }
                    }
                    util::set_property(scope, result, "headers", header.into());
                    let body = v8::String::new(scope, &body).unwrap();
                    util::set_property(scope, result, "body", body.into());
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

pub fn register(scope: &mut v8::PinScope, core: v8::Local<'_, v8::Object>) {
    let http = v8::String::new(scope, "http").unwrap();
    let obj = v8::Object::new(scope);
    util::set_method(scope, obj, "request", request);
    core.set(scope, http.into(), obj.into());
}
