import { filterRows, counts, appendPage, windowRange } from './model.mjs';

const $ = selector => document.querySelector(selector);
const $$ = selector => [...document.querySelectorAll(selector)];
const t = (key, params) => globalThis.causativelensI18n?.translate(key, params) || key;
let templates = [], selected = '', step = 0, workspace = 'sessions', probe = null;
let sessionId = null, rows = [], cursor = 0, filter = 'all', follow = true;
let connected = false, monitorStatus = null, timer = 0, epoch = 0, detail = null, lastFocus = null;
let bannerKey = null, lastUpdated = null;
const language = () => document.documentElement.lang;
const nameOf = id => templates.find(item => item.id === id)?.name || id;
function setTheme(theme) {
  document.documentElement.dataset.theme = theme;
  try { localStorage.setItem('causativelens-theme', theme); } catch { /* Optional storage. */ }
  drawGraph();
}
let initialTheme;
try { initialTheme = localStorage.getItem('causativelens-theme'); } catch { /* Optional storage. */ }
document.documentElement.dataset.theme = initialTheme || (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
$('#theme-toggle').addEventListener('click', () => setTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'));

async function invoke(command, args = {}) {
  if (!globalThis.__TAURI__?.core?.invoke) throw new Error('browser_preview');
  return globalThis.__TAURI__.core.invoke(command, args);
}
const rpc = (method, params = {}) => invoke('rpc_call', { method, params });
function banner(message, error = false, key = null) {
  bannerKey = key;
  $('#message-banner').textContent = message;
  $('#message-banner').classList.toggle('error', error);
  $('#message-banner').classList.toggle('hidden', !message);
}
function report(error) {
  const raw = String(error?.message || error);
  const key = raw.includes('browser_preview') ? 'error.preview' : /folder|directory|linked_folder/.test(raw) ? 'error.folder' : /session/.test(raw) ? 'error.session' : /transport|pipe|connect|daemon/.test(raw) ? 'error.connection' : 'error.operation';
  banner(t(key) + ' [' + raw.slice(0, 240) + ']', true);
}
function connectionState(value) {
  connected = value;
  $('#connection-status').classList.toggle('connected', value);
  $('#connection-status span').textContent = t(value ? 'connection.connected' : 'connection.offline');
}
function showWorkspace(name) {
  workspace = name;
  $$('[data-panel]').forEach(panel => panel.classList.toggle('hidden', panel.dataset.panel !== name));
  $$('.nav-item[data-workspace]').forEach(item => {
    item.classList.toggle('active', item.dataset.workspace === name);
    if (item.dataset.workspace === name) item.setAttribute('aria-current', 'page'); else item.removeAttribute('aria-current');
  });
  $('#page-label').textContent = t('nav.' + name);
  if (name === 'timeline') renderTimeline();
  if (name === 'graph') requestAnimationFrame(drawGraph);
}
$$('.nav-item[data-workspace]').forEach(item => item.addEventListener('click', () => showWorkspace(item.dataset.workspace)));
$('.brand').addEventListener('click', event => { event.preventDefault(); showWorkspace('sessions'); });
$('#timeline-setup').addEventListener('click', () => showWorkspace('sessions'));
$('#view-timeline').addEventListener('click', () => showWorkspace('timeline'));

function goStep(next) {
  step = next;
  $$('[data-step]').forEach(el => el.classList.toggle('hidden', Number(el.dataset.step) !== step));
  $$('[data-step-marker]').forEach(el => {
    const index = Number(el.dataset.stepMarker);
    el.classList.toggle('complete', index < step);
    if (index === step) el.setAttribute('aria-current', 'step'); else el.removeAttribute('aria-current');
  });
}
function chooseRuntime(value) {
  if (selected !== value) probe = null;
  selected = value;
  $('#runtime-select').value = value;
  $('#software-next').disabled = !value || !connected;
  $$('.runtime-shortcut').forEach(button => {
    button.classList.toggle('selected', button.dataset.runtime === value);
    button.setAttribute('aria-pressed', String(button.dataset.runtime === value));
  });
  const template = templates.find(item => item.id === value);
  $('#template-guidance').textContent = template?.guidance?.[language()] || template?.guidance?.['en-US'] || '';
}
function catalog(items) {
  templates = items;
  const option = document.createElement('option'); option.value = ''; option.textContent = t('software.placeholder');
  option.dataset.i18n = 'software.placeholder';
  $('#runtime-select').replaceChildren(option);
  for (const item of templates) {
    const option = document.createElement('option'); option.value = item.id; option.textContent = item.name;
    $('#runtime-select').append(option);
  }
  $('#runtime-select').disabled = false;
  const picks = document.createDocumentFragment();
  for (const id of ['hermes', 'claude-code', 'codex', 'openclaw', 'manus']) {
    const button = document.createElement('button');
    button.className = 'runtime-shortcut'; button.dataset.runtime = id; button.textContent = nameOf(id);
    button.addEventListener('click', () => chooseRuntime(id)); picks.append(button);
  }
  $('#featured-runtimes').replaceChildren(picks);
  chooseRuntime(selected);
}
$('#runtime-select').addEventListener('change', event => chooseRuntime(event.target.value));
$('#software-next').addEventListener('click', () => { goStep(1); $('#monitor-folder').focus(); });
$('#folder-back').addEventListener('click', () => goStep(0));
$('#review-back').addEventListener('click', () => goStep(1));
$('#monitor-folder').addEventListener('input', () => { probe = null; $('#probe-folder').disabled = !$('#monitor-folder').value.trim(); });
$('#browse-folder').addEventListener('click', async () => {
  try {
    const folder = await invoke('pick_monitor_folder');
    if (folder) { $('#monitor-folder').value = folder; probe = null; $('#probe-folder').disabled = false; }
  } catch (error) { report(error); }
});
function parameters() {
  return { runtime_id: selected, folder: $('#monitor-folder').value.trim(), include_profile: $('#include-profile').checked, read_existing: $('#read-existing').checked };
}
async function busy(button, work) {
  const key = button.dataset.i18n;
  button.disabled = true; button.textContent = t('busy');
  try { return await work(); } catch (error) { report(error); return null; }
  finally { button.disabled = false; if (key) button.textContent = t(key); showProbe(); renderMonitor(); }
}
function showProbe() {
  if (!probe) return;
  const recognized = probe.recognized.length > 0, files = probe.files.length;
  $('#probe-title').textContent = t(recognized ? 'probe.ready' : files ? 'probe.unknown' : 'probe.empty');
  $('#probe-description').textContent = t('probe.description', { roots: probe.roots.length, files });
  $('#review-runtime').textContent = nameOf(selected);
  $('#review-folder').textContent = $('#monitor-folder').value;
  const fragment = document.createDocumentFragment();
  for (const path of [...probe.roots, ...probe.files.slice(0, 20)]) {
    const li = document.createElement('li'); li.textContent = path; fragment.append(li);
  }
  $('#probe-files').replaceChildren(fragment);
  $('#start-monitor').textContent = t(files ? 'monitor.start' : 'monitor.waitStart');
  $('#start-monitor').disabled = Boolean(monitorStatus?.health?.running);
}
$('#probe-folder').addEventListener('click', () => busy($('#probe-folder'), async () => {
  probe = await rpc('monitor.probe', parameters());
  banner('');
  showProbe(); goStep(2);
}));
$('#start-monitor').addEventListener('click', () => busy($('#start-monitor'), async () => {
  if (!probe) return;
  if (sessionId && !monitorStatus?.health?.running) await rpc('session.close', { session_id: sessionId });
  const ticket = await rpc('monitor.start', parameters());
  sessionId = ticket.session_id; rows = []; cursor = 0; monitorStatus = null;
  epoch++; clearTimeout(timer);
  $('#active-monitor').classList.remove('hidden');
  banner(t('monitor.started'), false, 'monitor.started');
  await poll(epoch);
  $('#active-monitor').scrollIntoView({ block: 'nearest', behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth' });
}));
$('#stop-monitor').addEventListener('click', () => busy($('#stop-monitor'), async () => {
  if (!sessionId) return;
  monitorStatus = await rpc('monitor.stop', { session_id: sessionId });
  epoch++; clearTimeout(timer);
  await readPages(epoch);
  renderMonitor(); banner(t('monitor.stopped'), false, 'monitor.stopped');
}));
function renderMonitor() {
  const status = monitorStatus;
  if (!status) return;
  const health = status.health || {};
  $('#active-monitor').classList.remove('hidden');
  $('#active-runtime').textContent = nameOf(status.runtime_id);
  $('#active-folder').textContent = status.folder || '';
  $('#live-label').textContent = t(!health.running ? 'monitor.stopped' : health.recognized ? 'monitor.listening' : 'monitor.waiting');
  $('#live-label').classList.toggle('waiting', !health.recognized || !health.running);
  $('#stat-files').textContent = health.files || 0;
  $('#stat-records').textContent = health.recognized || 0;
  $('#stat-assertions').textContent = status.assertions || 0;
  $('#stat-rejected').textContent = (health.rejected || 0) + (health.io_errors || 0);
  const notes = [t('monitor.scope')];
  if (health.scan_limited || health.skipped_bytes || health.dropped) notes.push(t('monitor.truncated'));
  if (health.last_error) notes.push(String(health.last_error).slice(0, 180));
  $('#monitor-health').textContent = notes.join(' · ');
  $('#stop-monitor').disabled = !health.running;
  $('#start-monitor').disabled = Boolean(health.running);
  $('#timeline-source').textContent = nameOf(status.runtime_id) + ' · ' + t('monitor.scope');
  $('#footer-state').textContent = t(!health.running ? 'monitor.stopped' : health.recognized ? 'monitor.listening' : 'monitor.waiting');
}
async function readPages(generation) {
  const id = sessionId;
  let changed = false;
  for (let batch = 0; batch < 8; batch++) {
    const page = await rpc('monitor.events', { session_id: id, after: cursor, limit: 100 });
    if (generation !== epoch || sessionId !== id) return;
    if (cursor && page.retained_from > cursor + 1) banner(t('timeline.gap'));
    if (page.rows.length) { rows = appendPage(rows, page.rows); changed = true; }
    cursor = page.next_cursor;
    if (!page.has_more) break;
  }
  if (changed) {
    lastUpdated = new Date();
    $('#last-updated').textContent = t('timeline.updated', { time: lastUpdated.toLocaleTimeString(language()) });
    renderTimeline(); if (workspace === 'graph') drawGraph();
    if (follow && workspace === 'timeline') { $('#timeline-list').scrollTop = $('#timeline-list').scrollHeight; renderTimeline(); }
  }
}
async function poll(generation) {
  if (!sessionId || generation !== epoch) return;
  try {
    const status = await rpc('monitor.status', { session_id: sessionId });
    if (generation !== epoch) return;
    monitorStatus = status; connectionState(true);
    await readPages(generation);
    if (generation !== epoch) return;
    renderMonitor();
  } catch (error) { connectionState(false); report(error); }
  if (generation === epoch && monitorStatus?.health?.running !== false) timer = setTimeout(() => poll(generation), 500);
}
async function connectDaemon() {
  try {
    const hello = await invoke('connect_daemon');
    $('#version-label').textContent = hello.version;
    connectionState(true);
    catalog((await rpc('monitor.templates')).templates);
    const list = await rpc('monitor.list');
    const existing = list.monitors.find(item => item.health?.running) || list.monitors.at(-1);
    if (existing && (!sessionId || existing.session_id !== sessionId)) {
      sessionId = existing.session_id; rows = []; cursor = 0; monitorStatus = existing;
      epoch++; clearTimeout(timer); await poll(epoch);
    }
    banner('');
  } catch (error) { connectionState(false); report(error); }
}
$('#connection-status').addEventListener('click', connectDaemon);

let filtered = [];
function renderTimeline() {
  filtered = filterRows(rows, filter, $('#timeline-search').value);
  const totals = counts(rows);
  $$('[data-count]').forEach(el => { el.textContent = totals[el.dataset.count]; });
  $('#nav-count').textContent = rows.length;
  $('#visible-count').textContent = t('timeline.count', { count: filtered.length, total: rows.length });
  const empty = filtered.length === 0;
  $('#timeline-empty').classList.toggle('hidden', !empty);
  $('#timeline-empty h2').textContent = t(rows.length ? 'timeline.noMatch' : 'timeline.empty');
  $('#timeline-empty p').textContent = t(rows.length ? 'timeline.noMatchHelp' : 'timeline.emptyHelp');
  renderWindow();
}
function renderWindow() {
  const list = $('#timeline-list');
  const range = windowRange(filtered.length, list.scrollTop, list.clientHeight || 400);
  const fragment = document.createDocumentFragment();
  const spacer = height => { const node = document.createElement('div'); node.style.height = height + 'px'; node.setAttribute('aria-hidden', 'true'); return node; };
  fragment.append(spacer(range.top));
  const visibleRows = filtered.slice(range.start, range.end).slice(-500);
  for (const row of visibleRows) {
    const node = document.createElement('div'); node.className = 'timeline-row'; node.tabIndex = 0;
    node.setAttribute('role', 'button'); node.dataset.seq = row.seq; node.dataset.category = row.category;
    const time = document.createElement('span'); time.className = 'row-time';
    time.textContent = new Date(row.ts_utc_ms).toLocaleTimeString(language(), { hour12: false });
    const source = document.createElement('small'); source.textContent = t('source.' + row.source); time.append(source);
    const kind = document.createElement('span'); kind.className = 'row-kind'; kind.textContent = row.kind;
    const content = document.createElement('span'); content.className = 'row-content';
    const target = document.createElement('strong'); target.textContent = row.target || row.text || '—';
    const text = document.createElement('small'); text.textContent = row.text; content.append(target, text);
    const state = document.createElement('span'); state.className = 'row-state'; state.textContent = t('state.' + row.state);
    node.append(time, kind, content, state);
    node.addEventListener('click', () => showDetail(row, node));
    node.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); showDetail(row, node); } });
    fragment.append(node);
  }
  fragment.append(spacer(range.bottom));
  $('#timeline-rows').replaceChildren(fragment);
}
let scrollFrame = 0;
$('#timeline-list').addEventListener('scroll', () => {
  if (!scrollFrame) scrollFrame = requestAnimationFrame(() => { scrollFrame = 0; renderWindow(); });
});
$$('[data-filter]').forEach(button => button.addEventListener('click', () => {
  filter = button.dataset.filter;
  $$('[data-filter]').forEach(item => { item.classList.toggle('active', item === button); item.setAttribute('aria-pressed', String(item === button)); });
  $('#timeline-list').scrollTop = 0; renderTimeline();
}));
$('#timeline-search').addEventListener('input', () => { $('#timeline-list').scrollTop = 0; renderTimeline(); });
$('#follow-toggle').addEventListener('click', () => {
  follow = !follow; $('#follow-toggle').setAttribute('aria-pressed', String(follow));
  $('#follow-toggle').classList.toggle('selected', follow);
  if (follow) { $('#timeline-list').scrollTop = $('#timeline-list').scrollHeight; renderTimeline(); }
});
$('#export-timeline').addEventListener('click', () => {
  const blob = new Blob([JSON.stringify({ schema_v: 1, session_id: sessionId, mode: 'observe', capture_connected: false, rows }, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a'); anchor.href = url; anchor.download = 'CausativeLens-traces-' + Date.now() + '.json';
  anchor.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
});
function showDetail(row, trigger) {
  detail = row; lastFocus = trigger;
  $('#detail-content').textContent = JSON.stringify(row, null, 2);
  $('#detail-overlay').classList.remove('hidden'); $('#close-detail').focus();
}
function closeDetail() { $('#detail-overlay').classList.add('hidden'); lastFocus?.focus(); }
$('#close-detail').addEventListener('click', closeDetail);
$('#detail-overlay').addEventListener('click', event => { if (event.target === $('#detail-overlay')) closeDetail(); });
$('#copy-detail').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText(JSON.stringify(detail, null, 2)); banner(t('copied')); } catch (error) { report(error); }
});
document.addEventListener('keydown', event => {
  if (event.key === 'Escape') closeDetail();
  if (event.key === 'Tab' && !$('#detail-overlay').classList.contains('hidden')) {
    const first = $('#close-detail'), last = $('#copy-detail');
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }
});
function drawGraph() {
  const canvas = $('#graph-canvas');
  if (!canvas || workspace !== 'graph') return;
  const rect = canvas.getBoundingClientRect(), ratio = devicePixelRatio || 1;
  canvas.width = rect.width * ratio; canvas.height = rect.height * ratio;
  const ctx = canvas.getContext('2d'); ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.clearRect(0, 0, rect.width, rect.height);
  const assertions = rows.filter(row => row.category === 'assertions').slice(-60);
  $('#graph-empty').classList.toggle('hidden', Boolean(assertions.length));
  const styles = getComputedStyle(document.documentElement);
  assertions.forEach((row, index) => {
    const x = 45 + (index % 3) * Math.max(180, (rect.width - 90) / 3), y = 45 + Math.floor(index / 3) * 64;
    if (y > rect.height) return;
    ctx.fillStyle = styles.getPropertyValue('--accent'); ctx.beginPath(); ctx.arc(x, y, 5, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = styles.getPropertyValue('--text'); ctx.font = '11px Segoe UI';
    ctx.fillText(row.kind + ' · ' + row.target.slice(-24), x + 14, y + 4);
    ctx.fillStyle = styles.getPropertyValue('--muted'); ctx.font = '10px Segoe UI'; ctx.fillText(t('state.unobserved'), x + 14, y + 20);
  });
}
addEventListener('resize', () => { renderTimeline(); drawGraph(); });
$('#query-form').addEventListener('submit', event => {
  event.preventDefault();
  busy($('#run-query'), async () => {
    if (!sessionId) throw new Error('session_required');
    const result = await rpc('query.why', { session_id: sessionId, path: $('#query-input').value.trim() });
    $('#query-result').textContent = JSON.stringify(result.chain, null, 2);
  });
});
addEventListener('causativelens-language-changed', () => {
  connectionState(connected); chooseRuntime(selected); showProbe(); renderMonitor(); renderTimeline(); drawGraph();
  if (bannerKey) banner(t(bannerKey), false, bannerKey);
  if (lastUpdated) $('#last-updated').textContent = t('timeline.updated', { time: lastUpdated.toLocaleTimeString(language()) });
  $('#page-label').textContent = t('nav.' + workspace);
});
goStep(0); renderTimeline(); connectDaemon();
