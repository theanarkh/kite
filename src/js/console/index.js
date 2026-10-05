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
