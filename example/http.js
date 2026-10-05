const http = require('http');

async function runMain() {
    const res = await http.request('http://localhost:9999', { method: "POST", data: { hello: "world" } });
    console.log(JSON.stringify(res, null, 2));
}

runMain();