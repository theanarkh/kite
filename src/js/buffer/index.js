const {
    buffer,
} = core;

class Buffer extends Uint8Array {
    toString(encoding = 'UTF-8') {
        return buffer.fromUTF8(this);
    }

    static alloc(length) {
        return new Buffer(length);
    }

    static from(data) {
        const uint8Array = buffer.writeUTF8(data);
        return new Buffer(uint8Array.buffer, uint8Array.byteOffset, uint8Array.byteLength);
    }

    static toString(bytes) {
        return buffer.fromUTF8(bytes);
    }

    static concat(arr) {
        let len = 0;
        for (let i = 0; i < arr.length; i++) {
            len += arr[i].byteLength;
        }
        const result = Buffer.alloc(len);
        let index = 0;
        for (let i = 0; i < arr.length; i++) {
            const buffer = arr[i];
            for (let j = 0; j < buffer.byteLength; j++) {
                result[index++] = buffer[j];
            }
        }
        return result;
    }
}

module.exports = Buffer;
