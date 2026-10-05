const http = require('http');

http.createServer((req, res) => {
    console.log(req.method, req.url);
    let body = '';
    req.on('data', (chunk) => {
        body += chunk;
    });
    req.on('end', () => {
        console.log(body);
        res.end('hello world');
    });

}).listen(9999);
console.log('server is running on port 9999');
