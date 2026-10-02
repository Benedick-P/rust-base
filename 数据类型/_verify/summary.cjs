// 汇总数据类型验证样例的结果（编译报错 / 运行时 panic / 正常）
const fs = require('fs');
const path = require('path');
const dir = 'D:\\rust学习\\数据类型\\_verify\\cases';

const files = fs.readdirSync(dir).filter((f) => f.endsWith('.err.txt')).sort();
const rows = [];

for (const f of files) {
  const name = f.replace('.err.txt', '');
  const raw = fs.readFileSync(path.join(dir, f), 'utf8');
  const lines = raw.split(/\r?\n/);

  // 1. 运行时 panic
  const panicLine = lines.find((l) => /panicked at/.test(l));
  if (panicLine) {
    const msgLine = lines.find((l) =>
      /^\s*(attempt to |index out of bounds|no entry found|end byte index)/.test(l));
    rows.push({ name, kind: 'panic', code: 'panic', msg: (msgLine || panicLine).trim() });
    continue;
  }

  // 2. 编译错误 / 警告
  const head = lines.find((l) => /^(rustc : )?error(\[|:)/.test(l.trim()) && !/aborting/.test(l));
  if (head) {
    const code = (head.match(/\[(E\d+)\]/) || [])[1] || 'error';
    rows.push({ name, kind: 'error', code, msg: head.replace(/^rustc : /, '').trim() });
    continue;
  }

  // 3. 有输出但无错误 → 正常样例（可能有警告）
  const warn = lines.find((l) => /^(rustc : )?warning/.test(l.trim()));
  rows.push({
    name, kind: 'ok', code: '—',
    msg: warn ? '正常（有警告: ' + warn.replace(/^rustc : /, '').trim().slice(0, 40) + '）' : '正常，无输出',
  });
}

const kindLabel = { error: '编译错误', panic: '运行时 panic', ok: '正常' };
console.log('样例'.padEnd(30) + '类别'.padEnd(14) + '代号'.padEnd(9) + '信息');
console.log('-'.repeat(120));
for (const r of rows) {
  console.log(r.name.padEnd(26) + kindLabel[r.kind].padEnd(12) + String(r.code).padEnd(9) + r.msg.slice(0, 66));
}
const byKind = rows.reduce((a, r) => (a[r.kind] = (a[r.kind] || 0) + 1, a), {});
console.log('\n统计:', JSON.stringify(byKind), '共', rows.length, '个样例');

fs.writeFileSync(path.join(dir, '_summary.txt'),
  rows.map((r) => `${r.name}\t${kindLabel[r.kind]}\t${r.code}\t${r.msg}`).join('\n'), 'utf8');
console.log('已写入 cases/_summary.txt');
