mod rust;
use tokio;
use crate::rust::env;
use crate::rust::js;
use crate::rust::event_loop;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let local = tokio::task::LocalSet::new();

    rt.block_on(local.run_until(run()));
}

async fn run() {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
    // let cmds = vec![String::from("--expose-gc")];
    // v8::V8::set_flags_from_command_line(cmds);
    // 创建 Isolate 和 HandleScope
    let isolate = &mut v8::Isolate::new(Default::default());
    // 手动控制 Microtasks 执行
    isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
    let scope_storage = std::pin::pin!(v8::HandleScope::new(isolate));
    let mut handle_scope = scope_storage.init();

    // 创建 Context 和 ContextScope
    let context = v8::Context::new(&handle_scope, Default::default());
    let context_scope = &mut v8::ContextScope::new(&mut handle_scope, context);

    // 设置全局变量 global
    let global_key = v8::String::new(context_scope, "global").unwrap();
    let global = context.global(context_scope);
    global.set(context_scope, global_key.into(), global.into());

    // 创建 Env 和 EventLoop，设置到 Scope 中
    let env = env::Env::new();
    let event_loop = event_loop::EventLoop::new();
    context_scope.set_slot(env);
    context_scope.set_slot(event_loop);   

    // 注册模块给 JS 调用
    let core: v8::Local<'_, v8::Object> = v8::Object::new(context_scope);
    {
        let inner_scope = std::pin::pin!(v8::HandleScope::new(context_scope));
        let mut inner_scope = inner_scope.init();
        rust::register_modules(&mut inner_scope, core);
    }

    // 编译并执行 JS 代码
    let filename = "js/main.js";
    let code = js::get_js_code(filename).unwrap();
    let code = v8::String::new(context_scope, code).unwrap();
    let resource_name = v8::String::new(context_scope, filename).unwrap().into();   
    let origin = v8::ScriptOrigin::new(
        context_scope,
        resource_name,
        0,
        0,
        false,
        0,
        None,
        false,
        false,
        false,
        None,
    );
    let mut source = v8::script_compiler::Source::new(code, Some(&origin));
    let func = v8::script_compiler::compile_function(
        context_scope,
        &mut source,
        &[v8::String::new(context_scope, "core").unwrap()],
        &[],
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::NoReason,
    )
    .unwrap();
    let _ = func.call(context_scope, global.into(), &[core.into()]);

    // 开启 EventLoop
    event_loop::EventLoop::run(context_scope).await;
}
