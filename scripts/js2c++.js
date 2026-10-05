const fs = require('fs');
const path = require('path');

const libsPath = path.join(__dirname, '../js');
const dirs = fs.readdirSync(libsPath, { recursive: true });

let file;
const map = {};
while (file = dirs.shift()) {
    const fliePath = path.join(libsPath, file);
    const stat = fs.statSync(fliePath);
    if (stat.isFile()) {
        const content = fs.readFileSync(fliePath, 'utf-8');
        map[`js/${file}`] = content;
    }
}

function makeItem(file, content) {
    const indented = content
        .split('\n')
        .map(line => `            ${line}`)
        .join('\n');
    return `        "${file}" => Some(r#"\n${indented}\n        "#),`;
}

const contents = [];
for (const [file, content] of Object.entries(map)) {
    contents.push(makeItem(file, content));
}
const content = `pub fn get_js_code(name: &str) -> Option<&'static str> {
    match name {
${contents.join('\n')}
        _ => None,
    }
}
`;
fs.writeFileSync(path.join(__dirname, '../rust/js.rs'), content);