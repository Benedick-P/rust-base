// 生成「错误处理-思维导图.html」：以「包与模块-思维导图.html」为模板，换数据 + 换文案
const fs = require('fs');
const path = require('path');

const here = 'D:\\rust学习\\错误处理';
const target = path.join(here, '错误处理-思维导图.html');
const donor = path.join('D:\\rust学习\\包与模块', '包与模块-思维导图.html');
const mdPath = path.join(here, '错误处理-思维导图.mindmap.md');

if (!fs.existsSync(target)) {
  if (!fs.existsSync(donor)) { console.error('❌ 找不到模板:', donor); process.exit(1); }
  fs.copyFileSync(donor, target);
  console.log('已从模板复制:', path.basename(donor));
}

let html = fs.readFileSync(target, 'utf8');
const md = fs.readFileSync(mdPath, 'utf8');
const b64 = Buffer.from(md, 'utf8').toString('base64');

const re = /(<script id="mm-data" type="application\/octet-stream">)([\s\S]*?)(<\/script>)/;
if (!re.test(html)) { console.error('❌ 找不到 mm-data 数据块'); process.exit(1); }
html = html.replace(re, `$1${b64}$3`);

const subs = [
  [/<title>[^<]*<\/title>/, '<title>Rust 错误处理 · 思维导图</title>'],
  [/<span class="brand">[^<]*<\/span>/, '<span class="brand">RUST · ERROR HANDLING</span>'],
  [/<h1>[^<]*<\/h1>/, '<h1>Rust 错误处理 · panic 与 Result</h1>'],
  [/<code>包与模块-思维导图\.mindmap\.md<\/code>/, '<code>错误处理-思维导图.mindmap.md</code>'],
  [/'<code>包与模块-思维导图\.mindmap\.md<\/code>，'/, "'<code>错误处理-思维导图.mindmap.md</code>，'"],
  [/'Rust包与模块-思维导图\.svg'/, "'Rust错误处理-思维导图.svg'"],
  [/'Rust包与模块-思维导图\.png'/, "'Rust错误处理-思维导图.png'"],
];
let changed = 0;
for (const [r, rep] of subs) { if (r.test(html)) { html = html.replace(r, rep); changed++; } }

fs.writeFileSync(target, html, 'utf8');

if (Buffer.from(b64, 'base64').toString('utf8') !== md) { console.error('❌ base64 往返失败'); process.exit(1); }
const lines = md.split(/\r?\n/);
const h2 = lines.filter((l) => /^## /.test(l)).length;
const h3 = lines.filter((l) => /^### /.test(l)).length;
const li = lines.filter((l) => /^- /.test(l)).length;

console.log('✅ 已生成:', path.basename(target));
console.log('   文案替换:', changed, '/', subs.length);
console.log('   markdown 行数:', lines.length);
console.log('   章节:', h2, '| 小节:', h3, '| 叶子:', li, '| 总节点:', 1 + h2 + h3 + li);
console.log('   HTML 大小:', (fs.statSync(target).size / 1024).toFixed(1) + ' KB');

const final = fs.readFileSync(target, 'utf8');
const leftover = ['包与模块', '数据类型', '控制流', '所有权'].filter((b) => final.includes(b));
console.log(leftover.length ? '⚠️ 仍含旧主题字样: ' + leftover.join('、') : '✅ 无旧主题文案残留');
