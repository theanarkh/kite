const { Resolver } = require('dns');

async function main() {
    const resolver = new Resolver();
    const ips = await resolver.resolve4('localhost');
    console.log(JSON.stringify(ips));
}

main();