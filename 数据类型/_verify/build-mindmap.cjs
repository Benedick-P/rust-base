// 生成「数据类型-思维导图.html」：
// 1) 以「控制流-思维导图.html」为模板（结构相同，只换数据与文案）
// 2) 替换内嵌数据块
// 3) 替换标题 / 品牌 / 提示文案 / 导出文件名
const fs = require('fs');
const path = require('path');

const here = 'D:\\rust学习\\数据类型';
const target = path.join(here, '数据类型-思维导图.html');
const donor = path.join('D:\\rust学习\\控制流', '控制流-思维导图.html');
const mdPath = path.join(here, '数据类型-思维导图.mindmap.md');

if (!fs.existsSync(target)) {
  fs.copyFileSync(donor, target);
  console.log('已从模板复制:', path.basename(donor));
}

let html = fs.readFileSync(target, 'utf8');
const md = fs.readFileSync(mdPath, 'utf8');
const b64 = Buffer.from(md, 'utf8').toString('base64');

// 1. 数据块
const re = /(<script id="mm-data" type="application\/octet-stream">)([\s\S]*?)(<\/script>)/;
if (!re.test(html)) { console.error('❌ 找不到 mm-data 数据块'); process.exit(1); }
html = html.replace(re, `$1${b64}$3`);

// 2. 文案替换（幂等：无论当前是控制流还是数据类型的文案都能替换成功）
const subs = [
  [/<title>[^<]*<\/title>/, '<title>Rust 数据类型 · 思维导图</title>'],
  [/<span class="brand">[^<]*<\/span>/, '<span class="brand">RUST · DATA TYPES</span>'],
  [/<h1>[^<]*<\/h1>/, '<h1>Rust 数据类型 · 语法与注意事项</h1>'],
  [/<code>控制流-思维导图\.mindmap\.md<\/code>/, '<code>数据类型-思维导图.mindmap.md</code>'],
  [/'<code>控制流-思维导图\.mindmap\.md<\/code>，'/, "'<code>数据类型-思维导图.mindmap.md</code>，'"],
  [/'Rust控制流-思维导图\.svg'/, "'Rust数据类型-思维导图.svg'"],
  [/'Rust控制流-思维导图\.png'/, "'Rust数据类型-思维导图.png'"],
];
const changed = [];
for (const [re2, rep] of subs) {
  if (re2.test(html)) { html = html.replace(re2, rep); changed.push(String(re2).slice(0, 40)); }
}

fs.writeFileSync(target, html, 'utf8');

// 3. 自检
if (Buffer.from(b64, 'base64').toString('utf8') !== md) { console.error('❌ base64 往返失败'); process.exit(1); }
const lines = md.split(/\r?\n/);
const h2 = lines.filter((l) => /^## /.test(l)).length;
const h3 = lines.filter((l) => /^### /.test(l)).length;
const li = lines.filter((l) => /^- /.test(l)).length;

console.log('✅ 已生成:', path.basename(target));
console.log('   文案替换项数:', changed.length, '/', subs.length);
console.log('   markdown 行数:', lines.length);
console.log('   章节:', h2, '| 小节:', h3, '| 叶子:', li, '| 总节点:', 1 + h2 + h3 + li);
console.log('   HTML 大小:', (fs.statSync(target).size / 1024).toFixed(1) + ' KB');

// 4. 文案残留检查
const leftover = [];
if (/控制流/.test(fs.readFileSync(target, 'utf8'))) leftover.push('仍含「控制流」字样');
if (/所有权/.test(fs.readFileSync(target, 'utf8'))) leftover.push('仍含「所有权」字样');
console.log(leftover.length ? '⚠️ ' + leftover.join('，') : '✅ 无旧文案残留');
