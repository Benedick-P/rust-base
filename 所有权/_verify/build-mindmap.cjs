// 从「所有权与借用-思维导图.mindmap.md」重新生成自包含的交互式 HTML。
//
// 做法：以现有的「所有权与借用-思维导图.html」为模板（它自身就是模板），
//       只替换其中 <script id="mm-data"> 里的 base64 数据。
//
// 用法：node build-mindmap.cjs
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const htmlPath = path.join(root, '所有权与借用-思维导图.html');
const mdPath = path.join(root, '所有权与借用-思维导图.mindmap.md');

const html = fs.readFileSync(htmlPath, 'utf8');
const md = fs.readFileSync(mdPath, 'utf8');
const b64 = Buffer.from(md, 'utf8').toString('base64');

const re = /(<script id="mm-data" type="application\/octet-stream">)([\s\S]*?)(<\/script>)/;
if (!re.test(html)) {
  console.error('❌ 未在 HTML 中找到 <script id="mm-data"> 数据块，无法更新。');
  process.exit(1);
}

const before = html.match(re)[2].trim();
if (before === b64) {
  console.log('数据已是最新，无需修改。');
  console.log('  节点行数:', md.split(/\r?\n/).length);
  process.exit(0);
}

fs.writeFileSync(htmlPath, html.replace(re, `$1${b64}$3`), 'utf8');

// 自检：base64 必须能还原出原文
if (Buffer.from(b64, 'base64').toString('utf8') !== md) {
  console.error('❌ base64 往返失败');
  process.exit(1);
}

console.log('✅ 已更新:', htmlPath);
console.log('   markdown 行数:', md.split(/\r?\n/).length);
console.log('   HTML 大小:', (fs.statSync(htmlPath).size / 1024).toFixed(1) + ' KB');
console.log('   提示：页面里的 JS 逻辑（折叠按钮等）位于同一文件下方 <script> 中，可直接编辑。');
