'use strict';
const $ = id => document.getElementById(id);
const invoke = window.__TAURI__?.core.invoke;
let config, snapshot, busy = false, polling = false, saving = false, revision = 0;
const form = $('form');
function appearance(s) {
  document.documentElement.dataset.theme = s.theme_mode;
  document.documentElement.dataset.preset = s.color_preset;
}
function render(s) {
  snapshot = s;
  const seconds = s.remaining_seconds;
  $('time').textContent = String(Math.floor(seconds / 60)).padStart(2, '0') + ':' + String(seconds % 60).padStart(2, '0');
  $('phase').textContent = s.completed ? 'All cycles complete' : !s.running ? 'Ready to focus' : (s.phase === 'work' ? 'Focus time' : 'Take a break') + (s.paused ? ' · Paused' : '');
  $('caption').textContent = s.completed ? 'Well done. Take a moment for yourself.' : s.running ? (s.phase === 'work' ? 'One thing at a time.' : 'Step away. Breathe. Recharge.') : 'Make room for one thing at a time.';
  $('progress').value = s.progress;
  $('cycle').textContent = s.completed ? s.total + ' cycles completed' : s.running ? 'Cycle ' + s.cycle + ' of ' + s.total : s.total + ' focus cycles';
  $('start').disabled = busy || s.running;
  $('start').querySelector('.button-label').textContent = s.completed ? 'Start again' : 'Start focus';
  $('pause').disabled = busy || !s.running;
  $('pause').querySelector('.button-label').textContent = s.paused ? 'Resume' : 'Pause';
  $('pause-icon').setAttribute('d', s.paused ? 'm8 5 11 7-11 7z' : 'M8 5v14M16 5v14');
  $('reset').disabled = busy || (!s.running && !s.completed);
  $('cycles').disabled = busy || s.running;
}
async function action(action) {
  if (busy) return;
  const cycles = Number($('cycles').value);
  if (action === 'start' && (!Number.isInteger(cycles) || cycles < 1 || cycles > 99)) {
    $('error').textContent = 'Cycles must be a whole number between 1 and 99.'; return;
  }
  busy = true; revision++; $('error').textContent = ''; render(snapshot);
  try { render(await invoke('timer_action', { action, cycles })); }
  catch (e) { $('error').textContent = String(e); }
  finally { busy = false; render(snapshot); }
}
$('start').onclick = () => action('start');
$('pause').onclick = () => action(snapshot.paused ? 'resume' : 'pause');
$('reset').onclick = () => action('reset');
$('settings').onclick = async () => {
  try {
    config = await invoke('get_settings');
    for (const [key, value] of Object.entries(config)) {
      const field = form.elements.namedItem(key);
      if (!field) continue;
      if (field.type === 'checkbox') field.checked = value; else field.value = value;
    }
    $('settings-error').textContent = '';
    $('dialog').showModal();
  } catch (e) { $('error').textContent = String(e); }
};
function close() { if (!saving) $('dialog').close(); }
$('close').onclick = close; $('cancel').onclick = close;
$('dialog').addEventListener('cancel', e => { if (saving) e.preventDefault(); });
$('dialog').addEventListener('close', () => appearance(config));
function previewAppearance() {
  appearance({
    theme_mode: form.elements.namedItem('theme_mode').value,
    color_preset: form.elements.namedItem('color_preset').value,
  });
}
form.elements.namedItem('theme_mode').addEventListener('change', previewAppearance);
form.elements.namedItem('color_preset').addEventListener('change', previewAppearance);
form.onsubmit = async e => {
  e.preventDefault(); if (saving) return;
  const next = {};
  for (const key of Object.keys(config)) {
    const f = form.elements.namedItem(key);
    next[key] = f.type === 'checkbox' ? f.checked : f.type === 'number' ? Number(f.value) : f.value;
  }
  if (['work_minutes','break_minutes','cycles'].some(k => !Number.isInteger(next[k]))) {
    $('settings-error').textContent = 'Enter whole numbers for durations and cycles.'; return;
  }
  saving = true; $('save').disabled = true;
  try {
    await invoke('save_settings', { settings: next });
    config = next; appearance(config);
    if (!snapshot.running) $('cycles').value = config.cycles;
    $('routine').textContent = config.work_minutes + ' min focus · ' + config.break_minutes + ' min break';
    $('dialog').close();
    render(await invoke('get_state'));
  } catch (err) { $('settings-error').textContent = String(err); }
  finally { saving = false; $('save').disabled = false; }
};
async function poll() {
  if (polling || busy) return;
  polling = true;
  const requestedRevision = revision;
  try {
    const result = await invoke('get_state');
    if (!busy && revision === requestedRevision) render(result);
  }
  catch (e) { $('error').textContent = String(e); }
  finally { polling = false; }
}
async function init() {
  if (!invoke) { $('error').textContent = 'Open the desktop application to use the timer.'; return; }
  try {
    config = await invoke('get_settings'); appearance(config);
    $('cycles').value = config.cycles;
    $('routine').textContent = config.work_minutes + ' min focus · ' + config.break_minutes + ' min break';
    await poll(); setInterval(poll, 250);
  } catch(e) { $('error').textContent = String(e); }
}
init();
