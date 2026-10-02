// 控制流思维导图验收：渲染 + 折叠按钮 + 导出 PNG/SVG + H 键
const http = require('http');
const fs = require('fs');
const PORT = 9333;
function get(p) { return new Promise((res, rej) => { http.get({ host: '127.0.0.1', port: PORT, path: p }, r => { let d = ''; r.on('data', c => d += c); r.on('end', () => res(d)); }).on('error', rej); }); }
const sleep = ms => new Promise(r => setTimeout(r, ms));

(async () => {
  let targets = null;
  for (let i = 0; i < 40; i++) { try { targets = JSON.parse(await get('/json/list')); if (targets.some(t => t.type === 'page')) break; } catch (e) {} await sleep(250); }
  const page = targets.find(t => t.type === 'page');
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  let id = 0; const pending = new Map();
  ws.onmessage = ev => { const m = JSON.parse(ev.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); } };
  const send = (method, params) => new Promise(r => { const mid = ++id; pending.set(mid, r); ws.send(JSON.stringify({ id: mid, method, params: params || {} })); });
  const ev = async (expr) => {
    const r = await send('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true });
    if (r.result && r.result.exceptionDetails) return { __err: (r.result.exceptionDetails.exception || {}).description || r.result.exceptionDetails.text };
    return r.result && r.result.result ? r.result.result.value : null;
  };

  await send('Page.enable'); await send('Runtime.enable');
  await send('Page.navigate', { url: process.argv[2] });

  let ready = false;
  for (let i = 0; i < 60; i++) {
    await sleep(500);
    if (await ev(`document.querySelectorAll('#mindmap g.markmap-node').length > 5`)) { ready = true; break; }
  }
  if (!ready) { console.log('❌ 渲染失败'); process.exit(1); }
  await sleep(1000);

  // 1. 基本信息
  const info = await ev(`(() => ({
    标题: document.title,
    品牌: (document.querySelector('.brand') || {}).textContent,
    大标题: (document.querySelector('#hud h1') || {}).textContent,
    统计: (document.getElementById('stat') || {}).textContent,
    状态层已隐藏: document.getElementById('status').classList.contains('hidden'),
    节点数: document.querySelectorAll('#mindmap g.markmap-node').length,
    文本载体: document.querySelectorAll('#mindmap foreignObject').length + ' foreignObject'
  }))()`);
  console.log('=== 页面信息 ===');
  console.log(JSON.stringify(info, null, 2));

  // 2. 折叠按钮
  const seq = [];
  seq.push('初始=' + info.节点数);
  for (const [btn, label] of [['btn-l2', '展开二级'], ['btn-expand', '全部展开'], ['btn-collapse', '只看主干']]) {
    await ev(`document.getElementById('${btn}').click()`);
    await sleep(1400);
    seq.push(label + '=' + (await ev(`document.querySelectorAll('#mindmap g.markmap-node').length`)));
  }
  console.log('折叠按钮:', seq.join(' → '));

  // 3. 导出（拦截下载）
  await ev(`
    window.__out = null;
    if (!window.__patched) { window.__patched = true;
      const o = URL.createObjectURL; URL.createObjectURL = function (b) { window.__out = b; return o.call(URL, b); }; }
  `);
  await ev(`document.getElementById('btn-png').click()`);
  await sleep(4000);
  const png = await ev(`(async () => {
    const b = window.__out; if (!b) return '无数据';
    const u = URL.createObjectURL(b); const img = new Image();
    const ok = await new Promise(r => { img.onload = () => r(true); img.onerror = () => r(false); img.src = u; });
    if (!ok) { URL.revokeObjectURL(u); return { 类型: b.type, 大小KB: +(b.size/1024).toFixed(1), 加载: '失败' }; }
    const c = document.createElement('canvas'); c.width = 600; c.height = 400;
    const x = c.getContext('2d'); x.fillStyle = '#fff'; x.fillRect(0,0,600,400); x.drawImage(img, 0, 0, 600, 400);
    const d = x.getImageData(0,0,600,400).data; let n = 0;
    for (let i = 0; i < d.length; i += 4) if (d[i] < 250 || d[i+1] < 250 || d[i+2] < 250) n++;
    URL.revokeObjectURL(u);
    return { 类型: b.type, 大小KB: +(b.size/1024).toFixed(1), 尺寸: img.naturalWidth + 'x' + img.naturalHeight, 内容占比: (n/240000*100).toFixed(2) + '%' };
  })()`);
  console.log('=== 导出 PNG ==='); console.log(JSON.stringify(png, null, 2));

  await ev(`window.__out = null; document.getElementById('btn-svg').click();`);
  await sleep(900);
  const svg = await ev(`(async () => {
    const b = window.__out; if (!b) return '无数据';
    const t = await new Promise(r => { const fr = new FileReader(); fr.onload = () => r(String(fr.result)); fr.readAsText(b); });
    return { 大小KB: +(b.size/1024).toFixed(1), 有viewBox: /viewBox="/.test(t), 有style: /<style/.test(t) };
  })()`);
  console.log('=== 导出 SVG ==='); console.log(JSON.stringify(svg, null, 2));

  // 4. H 键
  const b1 = await ev(`document.getElementById('hud').className`);
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'h', code: 'KeyH', windowsVirtualKeyCode: 72 });
  await sleep(400);
  const b2 = await ev(`document.getElementById('hud').className`);
  console.log('H 键:', JSON.stringify(b1), '→', JSON.stringify(b2));
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'h', code: 'KeyH', windowsVirtualKeyCode: 72 });
  await sleep(300);

  // 5. 截图（展开二级）
  await ev(`document.getElementById('btn-l2').click()`);
  await sleep(1500);
  await ev(`document.getElementById('btn-fit').click()`);
  await sleep(1000);
  const shot = await send('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync('D:\\rust学习\\控制流\\_verify\\思维导图预览.png', Buffer.from(shot.result.data, 'base64'));
  console.log('✅ 截图已保存');
  ws.close(); process.exit(0);
})().catch(e => { console.error('FAILED:', e.message); process.exit(1); });
