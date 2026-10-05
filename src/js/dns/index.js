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