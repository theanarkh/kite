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