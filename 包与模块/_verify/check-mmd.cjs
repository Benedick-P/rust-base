// 校验控制流 mermaid mindmap 结构：缩进合法性、特殊字符、括号配对
const fs = require('fs');
const p = 'D:\\rust学习\\包与模块\\包与模块-思维导图.mmd';
const lines = fs.readFileSync(p, 'utf8').split(/\r?\n/);

const problems = [];
const nodeLines = [];
lines.forEach((raw, i) => {
  const no = i + 1;
  if (!raw.trim()) return;
  if (/^\s*%%/.test(raw)) return;             // 注释
  if (/\t/.test(raw)) problems.push(`L${no}: 含 Tab 缩进（应用空格）`);
  nodeLines.push({ no, indent: raw.match(/^ */)[0].length, text: raw.trim() });
});

nodeLines.forEach((n) => {
  if (n.indent % 2 !== 0) problems.push(`L${n.no}: 缩进 ${n.indent} 不是 2 的倍数`);
});

let prev = null;
nodeLines.forEach((n) => {
  const level = n.indent / 2;
  if (prev !== null && level > prev + 1) problems.push(`L${n.no}: 层级从 ${prev} 跳到 ${level}（不能跨级）`);
  prev = level;
});

const firstReal = lines.findIndex((l) => l.trim() && !/^\s*%%/.test(l));
if (!/^mindmap\s*$/.test(lines[firstReal].trim())) {
  problems.push(`L${firstReal + 1}: 第一个非注释行必须是 mindmap`);
}

nodeLines.forEach((n) => {
  if (n.text === 'mindmap') return;
  const isRootShape = /^root\s*[()]{2}.*[()]{2}$/.test(n.text);
  const plain = !/[[\]{}()]/.test(n.text);
  if (!isRootShape && !plain) problems.push(`L${n.no}: 括号形态不在白名单 → ${n.text}`);
  if (/["'`]/.test(n.text)) problems.push(`L${n.no}: 含引号 → ${n.text}`);
  if (/[:;]/.test(n.text)) problems.push(`L${n.no}: 含冒号/分号 → ${n.text}`);
  if (/#/.test(n.text)) problems.push(`L${n.no}: 含 # → ${n.text}`);
});

nodeLines.forEach((n) => {
  const o = (n.text.match(/\(/g) || []).length;
  const c = (n.text.match(/\)/g) || []).length;
  if (o !== c) problems.push(`L${n.no}: 圆括号不配对 → ${n.text}`);
});

const byLevel = {};
nodeLines.forEach((n) => { const l = n.indent / 2; byLevel[l] = (byLevel[l] || 0) + 1; });
const maxLevel = Math.max(...Object.keys(byLevel).map(Number));

console.log('总节点行数:', nodeLines.length);
console.log('各层级节点数:', JSON.stringify(byLevel));
console.log('最大层级:', maxLevel);
console.log('根节点:', nodeLines[1] ? nodeLines[1].text : '(缺失)');
console.log('章节数(=层级2):', byLevel[2] || 0);
console.log('');
if (problems.length) {
  console.log(`❌ 发现 ${problems.length} 个问题:`);
  problems.slice(0, 40).forEach((x) => console.log('  ' + x));
  process.exit(1);
} else {
  console.log('✅ Mermaid 结构校验通过（缩进、层级、特殊字符、括号配对均正常）');
}
