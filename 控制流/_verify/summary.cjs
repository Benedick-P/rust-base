// 汇总控制流报错样例的首条诊断信息
const fs = require('fs');
const path = require('path');
const dir = 'D:\\rust学习\\控制流\\_verify\\cases';

const files = fs.readdirSync(dir).filter((f) => f.endsWith('.err.txt')).sort();
const summary = [];

for (const f of files) {
  let raw = fs.readFileSync(path.join(dir, f), 'utf8');
  const lines = raw.split(/\r?\n/);
  // 第一行是 PowerShell 的 "rustc : error[...]: xxx" 包装
  const head = lines.find((l) => /^(rustc : )?(error|warning)/.test(l.trim())) || '';
  const code = (head.match(/\[(E\d+)\]/) || [])[1] || (head.match(/^rustc : (error|warning):/) || [])[1] || '';
  const msg = head.replace(/^rustc : /, '').trim();
  // 取主代码段（--> 之后的几行）
  const arrowIdx = lines.findIndex((l) => /^\s*-->/.test(l));
  const snippet = arrowIdx >= 0 ? lines.slice(arrowIdx, arrowIdx + 12).join('\n') : '(无代码段)';
  summary.push({ file: f.replace('.err.txt', ''), code, msg });
  if (process.argv[2] === '--verbose') {
    console.log('===== ' + f + ' =====');
    console.log(msg);
    console.log(snippet);
    console.log('');
  }
}

console.log('文件'.padEnd(34) + '错误码'.padEnd(10) + '信息');
console.log('-'.repeat(110));
for (const s of summary) {
  console.log(s.file.padEnd(30) + String(s.code).padEnd(10) + s.msg.slice(0, 70));
}
fs.writeFileSync('D:\\rust学习\\控制流\\_verify\\cases\\_summary.txt',
  summary.map((s) => `${s.file}\t${s.code}\t${s.msg}`).join('\n'), 'utf8');
