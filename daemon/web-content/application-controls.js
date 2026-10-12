const $ = id => document.getElementById(id);
let serial = '';
let saved = [];
let draft = [];
let running = [];
let runtime = {};
let routes = [];
let saving = false;
let polling = false;
let connected = false;

const channelNames = {Music:'Música',Game:'Jogo',Chat:'Chat',System:'Sistema',Console:'Consola',LineIn:'Line In',Sample:'Samples'};
const playbackChannels = ['Music', 'Game', 'Chat', 'System', 'Sample'];
function option(value, text) { const item = document.createElement('option'); item.value = value; item.textContent = text; return item; }
function message(text, error = false) { $('feedback').textContent = text; $('feedback').classList.toggle('error', error); }
function dirty() { return JSON.stringify(saved) !== JSON.stringify(draft); }
function nextSlot(channel, except) {
  let slot = 0;
  while (draft.some(app => app !== except && app.channel === channel && app.slot === slot)) slot++;
  return slot;
}
function setSlot(binding, slot) {
  const occupied = draft.find(app => app !== binding && app.channel === binding.channel && app.slot === slot);
  if (occupied) occupied.slot = binding.slot;
  binding.slot = slot;
  changed();
}
function renderAddPosition() {
  const slot = nextSlot($('app-channel').value);
  $('app-page').value = Math.floor(slot / 4) + 1;
  $('app-fader').value = 'ABCD'[slot % 4];
}
function controls() {
  $('device').disabled = saving || !serial;
  $('save').disabled = saving || !connected || !dirty();
  $('add-button').disabled = saving || !connected || !$('running-app').value;
}
function renderRunning() {
  const selected = $('running-app').value;
  const available = running.filter(app => !draft.some(binding => binding.executable.toLowerCase() === app.executable.toLowerCase()));
  $('running-app').replaceChildren(option('', available.length ? 'Escolher aplicação com som…' : 'Nenhuma aplicação nova detetada'));
  for (const app of available) $('running-app').append(option(app.executable, `${app.name} · ${app.volume}%${app.muted ? ' · sem som' : ''}`));
  if (available.some(app => app.executable === selected)) $('running-app').value = selected;
  $('add-button').disabled = saving || !connected || !$('running-app').value;
}
function renderSlots() {
  const container = $('page-slots'); container.replaceChildren();
  const apps = runtime.channel ? saved.filter(app => app.channel === runtime.channel) : [];
  const start = (runtime.page || 0) * 4;
  for (let i = 0; i < 4; i++) {
    const app = apps.find(app => app.slot === start + i); const slot = document.createElement('div'); slot.className = 'slot';
    if (app) slot.style.borderColor = app.colour;
    const title = document.createElement('strong'); title.textContent = `Slider ${'ABCD'[i]}${app ? ` · ${app.name}` : ''}`;
    const volume = runtime.volume?.[i];
    const muted = runtime.muted?.[i];
    const detail = document.createElement('span');
    detail.textContent = runtime.active
      ? `${runtime.pickup?.[i] ? `Move até ${volume ?? app?.reset_volume ?? 0}%` : `${volume ?? '—'}%`}${muted === true ? ' · sem som' : ''}`
      : (app ? `${channelNames[app.channel] || app.channel} · repor ${app.reset_volume}%` : 'Sem aplicação');
    slot.append(title, detail); container.append(slot);
  }
}
function renderRuntime() {
  if (!runtime.active) $('runtime').textContent = runtime.message || 'Modo normal · duplo clique num botão mute para abrir as aplicações do respetivo canal.';
  else $('runtime').textContent = `${channelNames[runtime.channel] || runtime.channel} · página ${runtime.page + 1} de ${runtime.page_count}. Beep recua, mute temporário avança.${runtime.message ? ` ${runtime.message}` : ''}`;
  renderSlots();
}
function renderApplications() {
  const list = $('applications'); list.replaceChildren();
  if (!draft.length) { const empty = document.createElement('p'); empty.className = 'subtle'; empty.textContent = 'Ainda não configuraste aplicações.'; list.append(empty); return; }
  draft.forEach((binding, index) => {
    const row = document.createElement('article'); row.className = 'application-row';
    const dot = document.createElement('span'); dot.className = 'colour-dot'; dot.style.background = binding.colour;
    const nameLabel = document.createElement('label'); nameLabel.textContent = 'Nome';
    const name = document.createElement('input'); name.value = binding.name; name.maxLength = 80; name.setAttribute('aria-label', `Nome da aplicação ${binding.name}`); name.oninput = () => { binding.name = name.value; changed(false); };
    nameLabel.append(name);
    const channelLabel = document.createElement('label'); channelLabel.textContent = 'Canal';
    const channel = document.createElement('select'); channel.setAttribute('aria-label', `Canal da aplicação ${binding.name}`);
    for (const id of playbackChannels) channel.append(option(id, channelNames[id]));
    if (!playbackChannels.includes(binding.channel)) channel.append(option(binding.channel, `${channelNames[binding.channel]} · entrada física`));
    channel.value = binding.channel; channel.onchange = () => { binding.channel = channel.value; binding.slot = nextSlot(binding.channel, binding); changed(); };
    channelLabel.append(channel);
    const pageLabel = document.createElement('label'); pageLabel.textContent = 'Página';
    const page = document.createElement('input'); page.type = 'number'; page.min = '1'; page.max = '100'; page.value = Math.floor(binding.slot / 4) + 1;
    page.setAttribute('aria-label', `Página de ${binding.name}`);
    page.onchange = () => { if (!page.checkValidity()) { page.reportValidity(); return; } setSlot(binding, (Number(page.value) - 1) * 4 + binding.slot % 4); }; pageLabel.append(page);
    const faderLabel = document.createElement('label'); faderLabel.textContent = 'Slider';
    const fader = document.createElement('select'); fader.setAttribute('aria-label', `Slider de ${binding.name}`);
    for (const id of 'ABCD') fader.append(option(id, id));
    fader.value = 'ABCD'[binding.slot % 4]; fader.onchange = () => setSlot(binding, Math.floor(binding.slot / 4) * 4 + 'ABCD'.indexOf(fader.value)); faderLabel.append(fader);
    const pathLabel = document.createElement('label'); pathLabel.textContent = 'Programa';
    const path = document.createElement('input'); path.value = binding.executable; path.readOnly = true; path.title = binding.executable; pathLabel.append(path);
    const colourLabel = document.createElement('label'); colourLabel.textContent = 'Cor';
    const colour = document.createElement('input'); colour.type = 'color'; colour.value = binding.colour; colour.setAttribute('aria-label', `Cor de ${binding.name}`); colour.oninput = () => { binding.colour = colour.value; dot.style.background = colour.value; changed(false); };
    colourLabel.append(colour);
    const resetLabel = document.createElement('label'); resetLabel.textContent = 'Volume de reposição (%)';
    const reset = document.createElement('input'); reset.type = 'number'; reset.min = '0'; reset.max = '100'; reset.value = binding.reset_volume; reset.setAttribute('aria-label', `Volume de reposição de ${binding.name}`); reset.oninput = () => { binding.reset_volume = Math.max(0, Math.min(100, Number(reset.value) || 0)); changed(false); };
    resetLabel.append(reset);
    const remove = document.createElement('button'); remove.type = 'button'; remove.className = 'remove'; remove.textContent = 'Remover'; remove.onclick = () => { draft.splice(index, 1); changed(); };
    const route = document.createElement('p'); route.className = 'route-status'; route.dataset.executable = binding.executable;
    row.append(dot, nameLabel, channelLabel, pageLabel, faderLabel, pathLabel, colourLabel, resetLabel, remove, route); list.append(row);
  });
  renderRoutes();
}
function renderRoutes() {
  document.querySelectorAll('.route-status').forEach(element => {
    const binding = draft.find(app => app.executable === element.dataset.executable);
    const persisted = saved.find(app => app.executable === binding.executable);
    const route = routes.find(item => item.executable.toLowerCase() === binding.executable.toLowerCase() && item.channel === binding.channel);
    element.classList.toggle('error', persisted?.channel === binding.channel && route?.state === 'error');
    element.textContent = !connected ? 'Liga o Mini para verificar o encaminhamento.' : (persisted?.channel !== binding.channel ? 'Guarda as alterações para encaminhar o áudio.' : (route?.message || 'A verificar o encaminhamento…'));
  });
}
function changed(render = true) {
  if (render) { renderApplications(); renderRunning(); }
  renderSlots();
  controls();
  message(dirty() ? 'Tens alterações por guardar.' : 'Configuração guardada.');
}
async function request(path, options) {
  const response = await fetch(path, {...options, signal: AbortSignal.timeout(6000)});
  if (!response.ok) throw new Error(await response.text() || 'O Utility recusou o pedido.');
  return response.json();
}
async function loadDevice() {
  const data = await request(`/api/application-controls/${encodeURIComponent(serial)}`);
  saved = structuredClone(data.applications || []); draft = structuredClone(saved);
  running = data.running_applications || []; runtime = data.runtime || {};
  routes = data.routes || [];
  renderApplications(); renderRunning(); renderRuntime();
  renderAddPosition();
  if (!data.discovery_available) message('Não foi possível consultar as aplicações com áudio do Windows.', true);
  else message(dirty() ? 'Tens alterações por guardar.' : 'Aplicações atualizadas.');
  controls();
}
async function poll() {
  if (polling) return;
  polling = true;
  try {
    const status = await request('/api/get-devices');
    const devices = Object.entries(status.mixers || {}).filter(([, mixer]) => mixer.hardware.device_type === 'Mini');
    const previousSerial = serial;
    if (!serial && devices.length) serial = devices[0][0];
    connected = devices.some(([id]) => id === serial);
    $('device').replaceChildren(...devices.map(([id]) => option(id, `GoXLR Mini · ${id.slice(-4)}`)));
    if (serial) $('device').value = serial;
    if (!serial && !devices.length) { message('Liga um GoXLR Mini para configurar e usar aplicações.'); controls(); return; }
    if (serial !== previousSerial) await loadDevice();
    else {
      const data = await request(`/api/application-controls/${encodeURIComponent(serial)}`);
      running = data.running_applications || []; runtime = data.runtime || {};
      routes = data.routes || [];
      renderRunning(); renderRuntime(); renderRoutes();
      if (!data.discovery_available) message('Não foi possível consultar as aplicações com áudio do Windows.', true);
    }
    controls();
  } catch (error) { connected = false; controls(); renderRoutes(); message(error.message || 'Não foi possível ligar ao Utility.', true); }
  finally { polling = false; }
}

$('running-app').onchange = () => {
  const app = running.find(item => item.executable === $('running-app').value);
  if (app) { $('app-name').value = app.name; $('reset-volume').value = app.volume; }
  controls();
};
$('add-form').onsubmit = event => {
  event.preventDefault();
  const app = running.find(item => item.executable === $('running-app').value);
  if (!app) return;
  const slot = (Number($('app-page').value) - 1) * 4 + 'ABCD'.indexOf($('app-fader').value);
  if (draft.some(binding => binding.channel === $('app-channel').value && binding.slot === slot)) { message('Esse slider já está ocupado nesta página e canal. Escolhe outro slider ou página.', true); return; }
  draft.push({name:$('app-name').value.trim(), executable:app.executable, channel:$('app-channel').value, colour:$('app-colour').value, reset_volume:Number($('reset-volume').value), slot});
  $('running-app').value = ''; $('app-name').value = '';
  renderApplications(); renderRunning(); renderSlots(); changed(false);
  renderAddPosition();
};
$('save').onclick = async () => {
  if (saving || !dirty()) return;
  saving = true; controls(); message('A guardar…');
  const submitted = structuredClone(draft);
  try {
    await request(`/api/application-controls/${encodeURIComponent(serial)}`, {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(submitted)});
    saved = submitted; routes = []; renderRoutes(); message(dirty() ? 'Configuração guardada. Tens novas alterações por guardar.' : 'Aplicações guardadas. O encaminhamento será verificado dentro de alguns segundos.');
  } catch (error) { message(error.message || 'Não foi possível guardar.', true); }
  finally { saving = false; controls(); }
};
$('device').onchange = async () => { if (dirty() && !confirm('Tens alterações por guardar. Queres descartá-las?')) { $('device').value = serial; return; } serial = $('device').value; await loadDevice(); };
$('app-channel').onchange = renderAddPosition;
poll(); setInterval(poll, 1500);
