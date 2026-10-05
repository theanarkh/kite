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