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