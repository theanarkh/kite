const net = require('net');

async function main() {
    try {
        const server = net.createServer({ address: '127.0.0.1:3000' });
        await server.listen();
        console.log('listen success');
        const conn = await server.accept();
        console.log(JSON.stringify(conn.get_addr()));
        while (true) {
            const buf = Buffer.alloc(1024);
            const nread = await conn.read(buf);
            if (nread === 0) {
                break;
            }
            console.log(buf.slice(0, nread).toString());
        }
        console.log('read end');
        await conn.write(Buffer.from('HTTP/1.1 200 OK\r\ncontent-length: 5\r\n\r\nhello\r\n\r\n'));
        console.log('write success');
    } catch (e) {
        console.log(e.message);
    }
}

main();
