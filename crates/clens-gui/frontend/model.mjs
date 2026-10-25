export function filterRows(rows, category = 'all', search = '') {
  const query = search.trim().toLocaleLowerCase();
  return rows.filter(row => (category === 'all' || row.category === category) && (!query || [row.target, row.text, row.kind, row.source].some(value => String(value || '').toLocaleLowerCase().includes(query))));
}
export function counts(rows) {
  const total = { all: rows.length, assertions: 0, files: 0, network: 0 };
  for (const row of rows) if (row.category in total && row.category !== 'all') total[row.category]++;
  return total;
}
export function appendPage(rows, page) {
  const merged = new Map(rows.map(row => [row.seq, row]));
  for (const row of page) merged.set(row.seq, row);
  return [...merged.values()].sort((a, b) => a.seq - b.seq).slice(-5000);
}
export function windowRange(length, scrollTop, height, rowHeight = 56) {
  const start = Math.min(length, Math.max(0, Math.floor(scrollTop / rowHeight) - 8));
  const end = Math.min(length, start + Math.min(500, Math.ceil(height / rowHeight) + 16));
  return { start, end, top: start * rowHeight, bottom: Math.max(0, (length - end) * rowHeight) };
}
