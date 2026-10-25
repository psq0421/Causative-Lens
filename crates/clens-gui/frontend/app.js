const panels = [...document.querySelectorAll('[data-panel]')];
const navItems = [...document.querySelectorAll('.nav-item[data-workspace]')];
const connection = document.querySelector('#connection-status');
const palette = document.querySelector('#command-palette');
const paletteInput = document.querySelector('#palette-input');
const prefersDark = matchMedia('(prefers-color-scheme: dark)');
let sessionId = null;
const i18n = globalThis.causativelensI18n;
const t = key => i18n?.translate(key) || key;

function showWorkspace(name) {
  panels.forEach(panel => panel.classList.toggle('hidden', panel.dataset.panel !== name));
  navItems.forEach(item => item.classList.toggle('active', item.dataset.workspace === name));
}
function setTheme(theme) { document.documentElement.dataset.theme = theme; localStorage.setItem('causativelens-theme', theme); }
setTheme(localStorage.getItem('causativelens-theme') || (prefersDark.matches ? 'dark' : 'light'));
document.querySelector('#theme-toggle').addEventListener('click', () => setTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'));
navItems.forEach(item => item.addEventListener('click', () => showWorkspace(item.dataset.workspace)));
function closePalette() { palette.classList.add('hidden'); }
function openPalette() { palette.classList.remove('hidden'); paletteInput.value = ''; paletteInput.focus(); }
document.querySelector('#command-search').addEventListener('click', openPalette);
palette.addEventListener('click', event => { if (event.target === palette) closePalette(); });
document.querySelectorAll('.palette-items button').forEach(item => item.addEventListener('click', () => { showWorkspace(item.dataset.action); closePalette(); }));
document.addEventListener('keydown', event => { if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') { event.preventDefault(); openPalette(); } if (event.key === 'Escape') closePalette(); });

function invoke(name, args) { return globalThis.__TAURI__?.core?.invoke(name, args); }
addEventListener('causativelens-language-changed', () => {
  const connected = connection.classList.contains('connected');
  connection.querySelector('span').textContent = t(connected ? 'connection.connected' : 'connection.offline');
  document.querySelector('#session-health').textContent = t(connected ? 'connection.connected' : 'connection.offline');
});
async function connectDaemon() {
  try {
    const ticket = await invoke('rpc_call', { method: 'session.open', params: {} });
    sessionId = ticket?.session_id || null;
    const result = await invoke('rpc_call', { method: 'session.status', params: sessionId ? { session_id: sessionId } : {} });
    if (result) {
      connection.classList.add('connected');
      connection.querySelector('span').textContent = t('connection.connected');
      document.querySelector('#session-health').textContent = t('connection.connected');
      return result;
    }
  } catch (_) { /* Offline mode is a supported read-only state. */ }
  connection.querySelector('span').textContent = t('connection.offline');
  document.querySelector('#session-health').textContent = t('connection.offline');
  return null;
}
function addTimelineRows(events) {
  const list = document.querySelector('#timeline-list');
  const fragment = document.createDocumentFragment();
  events.slice(-500).forEach((event, index) => {
    const row = document.createElement('div');
    row.className = 'timeline-row';
    row.innerHTML = `<span>${String(index + 1).padStart(4, '0')}</span><span class="kind">${event.kind || 'event'}</span><span>${event.text || event.target || 'observe'}</span><span class="state">${event.state || 'observed'}</span>`;
    fragment.appendChild(row);
  });
  list.replaceChildren(fragment);
}
class GraphDeltaRenderer {
  constructor(canvas) { this.canvas = canvas; this.ctx = canvas.getContext('2d'); this.nodes = new Map(); this.pending = []; this.timer = 0; addEventListener('resize', () => this.resize()); this.resize(); }
  resize() { const ratio = devicePixelRatio || 1; const rect = this.canvas.getBoundingClientRect(); this.canvas.width = rect.width * ratio; this.canvas.height = rect.height * ratio; this.ctx.setTransform(ratio, 0, 0, ratio, 0, 0); this.draw(); }
  push(delta) { this.pending.push(...delta); if (!this.timer) this.timer = setTimeout(() => { const batch = this.pending.splice(0); batch.forEach(node => this.nodes.set(node.id, node)); this.timer = 0; this.draw(); }, 100); }
  draw() { const rect = this.canvas.getBoundingClientRect(); const ctx = this.ctx; ctx.clearRect(0, 0, rect.width, rect.height); const values = [...this.nodes.values()]; values.forEach((node, index) => { const x = 80 + (index % 5) * Math.max(120, (rect.width - 160) / 5); const y = 90 + Math.floor(index / 5) * 90; ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--accent'); ctx.beginPath(); ctx.arc(x, y, 7, 0, Math.PI * 2); ctx.fill(); ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--text'); ctx.font = '12px Segoe UI'; ctx.fillText(node.label || 'assertion', x + 14, y + 4); }); }
}
const renderer = new GraphDeltaRenderer(document.querySelector('#graph-canvas'));
renderer.push([{ id: 'empty', label: 'observe graph' }]);
document.querySelector('#copy-run').addEventListener('click', () => navigator.clipboard?.writeText('clens run -- <agent-command>'));
document.querySelector('#copy-why').addEventListener('click', () => navigator.clipboard?.writeText('clens why --path <path>'));
document.querySelector('#open-session').addEventListener('click', () => showWorkspace('timeline'));
document.querySelector('#run-query').addEventListener('click', async () => {
  const text = document.querySelector('#query-input').value.trim();
  const pathMatch = text.match(/[A-Za-z]:\\[^?\s]+|[\w.-]+\.ya?ml/i);
  if (pathMatch) {
    try {
      const result = await invoke('rpc_call', { method: 'query.why', params: { session_id: sessionId, path: pathMatch[0] } });
      const evidence = result?.chain?.evidence_events || [];
      renderer.push([{ id: `query-${Date.now()}`, label: `${pathMatch[0]} (${evidence.length})` }]);
    } catch (_) { /* Keep the query surface deterministic and read-only. */ }
  }
  showWorkspace('graph');
});
document.querySelector('#help-button').addEventListener('click', openPalette);
connectDaemon();
