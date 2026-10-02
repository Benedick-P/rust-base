// 用「控制流-思维导图.mindmap.md」替换交互式 HTML 里的数据块。
// 模板取自同目录已生成好的 HTML（若不存在则从 ../所有权/ 复制一份）。
const fs = require('fs');
const path = require('path');

const here = 'D:\\rust学习\\控制流';
const target = path.join(here, '控制流-思维导图.html');
const donor = path.join('D:\\rust学习\\所有权', '所有权与借用-思维导图.html');
const mdPath = path.join(here, '控制流-思维导图.mindmap.md');

const re = /(<script id="mm-data" type="application\/octet-stream">)([\s\S]*?)(<\/script>)/;

// 1. 准备模板：没有就从「所有权」目录借一份（结构完全相同，只换数据）
if (!fs.existsSync(target)) {
  if (!fs.existsSync(donor)) {
    console.error('❌ 找不到模板：', donor);
    process.exit(1);
  }
  fs.copyFileSync(donor, target);
  console.log('已从模板复制:', path.basename(donor), '→', path.basename(target));
}

// 2. 替换数据块
const html = fs.readFileSync(target, 'utf8');
const md = fs.readFileSync(mdPath, 'utf8');
const b64 = Buffer.from(md, 'utf8').toString('base64');

if (!re.test(html)) {
  console.error('❌ HTML 中没有找到 <script id="mm-data"> 数据块');
  process.exit(1);
}

fs.writeFileSync(target, html.replace(re, `$1${b64}$3`), 'utf8');

// 3. 自检
const back = Buffer.from(b64, 'base64').toString('utf8');
if (back !== md) { console.error('❌ base64 往返失败'); process.exit(1); }

const lines = md.split(/\r?\n/);
const h2 = lines.filter((l) => /^## /.test(l)).length;
const h3 = lines.filter((l) => /^### /.test(l)).length;
const li = lines.filter((l) => /^- /.test(l)).length;

console.log('✅ 已更新:', path.basename(target));
console.log('   markdown 行数:', lines.length);
console.log('   章节数:', h2, '| 小节数:', h3, '| 叶子条目:', li, '| 总节点:', 1 + h2 + h3 + li);
console.log('   HTML 大小:', (fs.statSync(target).size / 1024).toFixed(1) + ' KB');
