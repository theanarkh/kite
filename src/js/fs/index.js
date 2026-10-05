module.exports = {
    read: (path) => {
        return core.fs.read(path);
    },
    write: (path, data) => {
        return core.fs.write(path, data);
    },
}