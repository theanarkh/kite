const { UDP } = require('udp');

async function server() {
    const udp = new UDP();
    await udp.bind('127.0.0.1:8888');
    console.log('udp is running on port 8888');
    return udp;
}

async function client() {
    const udp = new UDP();
    try {
        await udp.connect('127.0.0.1:8888');
    } catch (e) {
        console.log(`connect error: ${e.message}`);
        return;
    }
    console.log('udp is connected to 127.0.0.1:8888');
    const buf = Buffer.from('hello world');
    try {
        console.log('send hello world');
        await udp.send(buf, '127.0.0.1:8888');
        console.log('udp sent hello world to 127.0.0.1:8888');
    } catch (e) {
        console.log(`send error: ${e.message}`);
    }
}

server().then(async (udp) => {
    await client();
    // await udp.close();
    // gc();
    while (true) {
        const buf = Buffer.alloc(1024);
        const { nread, addr } = await udp.recv(buf);
        console.log(`from ${addr}: ${buf.slice(0, nread).toString()}`);
    }
});
