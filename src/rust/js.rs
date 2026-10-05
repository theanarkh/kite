pub fn get_js_code(name: &str) -> Option<&'static str> {
    match name {
        "js/main.js" => Some(r#"
            const {
                loader,
                process,
                console,
            } = core;
            
            const cache = {};
            
            class BuiltModule {
                exports = {};
            
                constructor(filename) {
                    this.filename = filename;
                }
            
                require() {
                    const result = loader.internalCompile(this.filename);
                    result.call(null, BuiltModule.require, this.exports, this, this.filename, this.filename, core);
                    return this.exports;
                }
            
                static require(filename) {
                    if (cache[filename]) {
                        return cache[filename];
                    }
                    const module = new BuiltModule(`js/${filename}/index.js`);
                    return (cache[filename] = module.require());
                }
            }
            
            function runMain() {
                const Buffer = BuiltModule.require("buffer");
                const { console } = BuiltModule.require("console");
                const { Module } = BuiltModule.require("module");
                
                global.console = console;
                global.Buffer = Buffer;
            
                let entry;
                for (let i = 0; i < process.argv.length; i++) {
                    if (process.argv[i].endsWith('.js')) {
                        entry = process.argv[i];
                        break;
                    }
                }
                if (!entry) {
                    console.log("No entry point found");
                    return;
                }
                if (process.isMainThread) {
                    Module.require(entry);
                } else {
                    Module.require("lib/worker/main.js");
                    Module.require(entry);
                }
            }
            
            runMain();
        "#),
        "js/buffer/index.js" => Some(r#"
            const {
                buffer,
            } = core;
            
            class Buffer extends Uint8Array {
                toString(encoding = 'UTF-8') {
                    return buffer.fromUTF8(this);
                }
            
                static alloc(length) {
                    return new Buffer(length);
                }
            
                static from(data) {
                    const uint8Array = buffer.writeUTF8(data);
                    return new Buffer(uint8Array.buffer, uint8Array.byteOffset, uint8Array.byteLength);
                }
            
                static toString(bytes) {
                    return buffer.fromUTF8(bytes);
                }
            
                static concat(arr) {
                    let len = 0;
                    for (let i = 0; i < arr.length; i++) {
                        len += arr[i].byteLength;
                    }
                    const result = Buffer.alloc(len);
                    let index = 0;
                    for (let i = 0; i < arr.length; i++) {
                        const buffer = arr[i];
                        for (let j = 0; j < buffer.byteLength; j++) {
                            result[index++] = buffer[j];
                        }
                    }
                    return result;
                }
            }
            
            module.exports = Buffer;
            
        "#),
        "js/console/index.js" => Some(r#"
            const { console } = core;
            
            class Console {
                constructor() {
                }
                log(msg) {
                    console.log(msg);
                }
            }
            
            module.exports = {
                Console,
                console: new Console(),
            };
            
        "#),
        "js/dns/index.js" => Some(r#"
            const { dns } = core;
            
            class Resolver {
                constructor() {
                    this.resolver = new dns.Resolver();
                }
            
                async resolve4(host) {
                    const ips = await this.resolver.resolve4(host);
                    return ips;
                }
            }
            
            module.exports = {
                Resolver,
            }
        "#),
        "js/fs/index.js" => Some(r#"
            module.exports = {
                read: (path) => {
                    return core.fs.read(path);
                },
                write: (path, data) => {
                    return core.fs.write(path, data);
                },
            }
        "#),
        "js/http/index.js" => Some(r#"
            const { http } = core;
            
            function get(url) {
                return http.request(url, { method: "GET" });
            }
            
            function request(url, options) {
                return http.request(url, options);
            }
            
            module.exports = {
                get,
                request,
            };
        "#),
        "js/module/index.js" => Some(r#"
            const {
                loader,
            } = core;
            
            const modeuls = {};
            const set = new Set(['fs', 'net', 'http', 'udp', 'dns']);
            
            class Module {
                constructor(filename) {
                    this.filename = filename;
                    this.exports = {};
                }
                require() {
                    const result = loader.compile(this.filename);
                    result.call(this, Module.require, this.exports, this, this.filename, this.filename);
                    return this.exports;
                }
                static require(filename, ...args) {
                    if (set.has(filename)) {
                        return require(filename);
                    }
                    if (!/\.js$/.test(filename)) {
                        filename = `${filename}.js`;
                    }
                    if (modeuls[filename]) {
                        return modeuls[filename];
                    }
                    const module = new Module(filename, ...args);
                    return (modeuls[filename] = module.require());
                }
            };
            
            module.exports = {
                Module,
            };
        "#),
        "js/net/index.js" => Some(r#"
            const {
                net
            } = core;
            
            class TCP {
                constructor(options) {
                    this.options = options;
                    this.tcp = new net.TCP();
                }
            
                async listen() {
                    await this.tcp.bind(this.options.address);
                }
            
                async accept() {
                    const conn = await this.tcp.accept();
                    return new Connection(conn);
                }
            }
            
            class Connection {
                constructor(socket) {
                    this.socket = socket;
                }
            
                get_addr() {
                    return this.socket.get_addr();
                }
            
                async read(buf) {
                    const nread = await this.socket.read(buf);
                    return nread;
                }
            
                async write(buf) {
                    const nwrite = await this.socket.write(buf);
                    return nwrite;
                }
            }
            
            function createServer(options) {
                return new TCP(options);
            }
            
            module.exports = {
                createServer,
            };
        "#),
        "js/udp/index.js" => Some(r#"
            const { udp } = core;
            
            const FLAG = {
                BIND: 0x01,
                CONNECT: 0x02,
            };
            
            class UDP {
                flag = 0;
                constructor() {
                    this.udp = new udp.UDP();
                }
            
                async bind(address) {
                    await this.udp.bind(address);
                    this.flag |= FLAG.BIND;
                }
            
                async connect(address) {
                    if (!(this.flag & FLAG.BIND)) {
                        await this.bind('0.0.0.0:0');
                    }
                    await this.udp.connect(address);
                    this.flag |= FLAG.CONNECT;
                }
            
                async recv(buf) {
                    return await this.udp.recv(buf);
                }
            
                async send(buf, address) {
                    // if (!(this.flag & FLAG.BIND)) {
                    //     await this.bind('0.0.0.0:0');
                    // }
                    if (this.flag & FLAG.CONNECT) {
                        return await this.udp.send(buf);
                    } else {
                        return await this.udp.send(buf, address);
                    }
                }
            
                async close() {
                    await this.udp.close();
                    this.udp = null;
                }
            }
            
            module.exports = {
                UDP,
            };
        "#),
        _ => None,
    }
}
