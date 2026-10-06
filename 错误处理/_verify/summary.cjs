// 汇总错误处理用例结果：读 cases/_raw.tsv + 各 diagnostic.txt
const fs = require('fs');
const path = require('path');
const dir = 'D:\\rust学习\\错误处理\\_verify\\cases';

const KIND_CN = { compile_error: '编译错误', panic: '运行时 panic', ok: '正常运行' };

function readTextSmart(file) {
  const buf = fs.readFileSync(file);
  if (buf.length >= 2 && buf[0] === 0xff && buf[1] === 0xfe) return buf.slice(2).toString('utf16le');
  let s = buf.toString('utf8');
  if (s.charCodeAt(0) === 0xfeff) s = s.slice(1);
  return s;
}

const rawPath = path.join(dir, '_raw.tsv');
if (!fs.existsSync(rawPath)) { console.error('缺少 _raw.tsv，请先运行 capture.ps1'); process.exit(1); }
const raw = readTextSmart(rawPath).split(/\r?\n/).filter((l) => l.trim());

const rows = [];
for (const line of raw) {
  const [name, kind, code] = line.split('\t');
  const f = path.join(dir, name, 'diagnostic.txt');
  let msg = '';
  let help = '';
  if (fs.existsSync(f)) {
    const lines = readTextSmart(f).split(/\r?\n/);
    if (kind === 'compile_error') {
      const head = lines.find((l) => /error(\[|:)/.test(l) && !/aborting/.test(l)) || '';
      msg = head.replace(/^.*?error/, 'error').trim();
      const h = lines.find((l) => /^(help|= help|= note)\s*:/.test(l.trim()));
      help = h ? h.trim() : '';
    } else if (kind === 'panic') {
      // panic 信息在 "panicked at ...:" 的下一行
      const idx = lines.findIndex((l) => /panicked at/.test(l));
      msg = idx >= 0 && lines[idx + 1] ? lines[idx + 1].trim() : '';
      if (!msg) msg = (lines.find((l) => /^\s*(called |attempt to|index out|assertion)/.test(l)) || '').trim();
      msg = msg.replace(/^note:.*/, '');
    } else {
      msg = '(无报错)';
    }
  }
  rows.push({ name, kind, code, msg, help });
}

console.log('用例'.padEnd(28) + '类别'.padEnd(12) + '代码'.padEnd(10) + '信息');
console.log('-'.repeat(150));
for (const r of rows) {
  console.log(r.name.padEnd(24) + (KIND_CN[r.kind] || r.kind).padEnd(10) + String(r.code).padEnd(10) + r.msg.slice(0, 88));
}

const byKind = rows.reduce((a, r) => ((a[KIND_CN[r.kind] || r.kind] = (a[KIND_CN[r.kind] || r.kind] || 0) + 1), a), {});
console.log('\n统计:', JSON.stringify(byKind), '共', rows.length, '个用例');
console.log('\n编译器给出的 help 摘要:');
for (const r of rows.filter((x) => x.help)) {
  console.log('  ' + r.name.padEnd(24) + r.help.slice(0, 90));
}

fs.writeFileSync(path.join(dir, '_summary.txt'),
  rows.map((r) => `${r.name}\t${KIND_CN[r.kind] || r.kind}\t${r.code}\t${r.msg}${r.help ? ' || ' + r.help : ''}`).join('\n'), 'utf8');
console.log('\n已写入 cases/_summary.txt');
