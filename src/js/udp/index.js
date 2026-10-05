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