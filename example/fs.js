async function main() {
    console.log(__filename);
    try {
        const data = await require('fs').read(__filename);
        console.log(data);
    } catch (e) {
        console.log(e.message);
    }
}

main();
