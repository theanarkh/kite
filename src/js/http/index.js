const { http } = core;

function get(url) {
    return http.request(url, { method: "GET" });
}

function request(url, options) {
    return http.request(url, options);
}

module.exports = {
    get,
    request,
};