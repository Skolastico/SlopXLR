const labels = ['Botão A', 'Botão B', 'Botão C', 'Botão D', 'Bleep'];
const kinds = [['none', 'Sem ação'], ['media_key', 'Controlar música / volume'], ['shortcut', 'Atalho de teclado'], ['load_profile', 'Carregar perfil'], ['toggle_mute', 'Alternar mute de um fader'], ['open_program', 'Abrir aplicação']];
const media = [[179, 'Play / pause'], [176, 'Faixa seguinte'], [177, 'Faixa anterior'], [178, 'Parar'], [173, 'Silenciar áudio do Windows'], [174, 'Diminuir volume do Windows'], [175, 'Aumentar volume do Windows']];
const $ = id => document.getElementById(id);
let saved, serial = '', available = false, dirty = false, saving = false, recording = null, polling = false, status, testing = null, lastReport = '';
const palette = ['#b06cff','#00c8c8','#ffa348','#75db68','#ed729f','#649aff'];
const newPage = (name, index = 0) => ({name, actions:Array.from({length:5}, () => ({kind:'none'})), colours:Array(6).fill(palette[index % palette.length]), pressed_colours:Array(6).fill('#ffffff')});
let pages = [newPage('Página 1')], selectedPage = 0, actions = pages[0].actions;

function message(text, error = false) { $('message').textContent = text; $('message').classList.toggle('error', error); }
function controls() {
  const busy = saving || testing !== null;
  $('enabled').disabled = !available || busy;
  document.querySelectorAll('.page-editor input, .page-editor select, .page-editor button').forEach(el => { el.disabled = !available || busy || recording !== null; });
  $('remove-page').disabled = !available || busy || recording !== null || pages.length < 2;
  $('save').disabled = !available || !dirty || busy || recording !== null;
  $('reset').disabled = !dirty || busy;
  $('device').disabled = !available || dirty || busy;
  document.querySelectorAll('#rows input, #rows select, #rows button').forEach(el => { el.disabled = !available || busy; });
}
function changed() { dirty = true; message('Alterações por guardar.'); controls(); }
function option(value, text) { const node = document.createElement('option'); node.value = value; node.textContent = text; return node; }
function select(values, value, label) {
  const node = document.createElement('select'); node.setAttribute('aria-label', label);
  for (const [key, text] of values) node.append(option(key, text));
  node.value = value;
  return node;
}
function describeKeys(keys) {
  return keys.map(key => ({16:'Shift',17:'Ctrl',18:'Alt',91:'Windows'})[key] || (key >= 112 ? `F${key - 111}` : String.fromCharCode(key))).join(' + ');
}
function cancelRecording() { recording = null; render(); controls(); }
function detailFor(index, parent) {
  const action = actions[index];
  const detail = document.createElement('div'); detail.className = 'action-detail';
  const label = document.createElement('label'); label.htmlFor = `value-${index}`;
  let input;
  if (action.kind === 'none') {
    const hint = document.createElement('p'); hint.textContent = 'Nesta página, este botão não executa nenhuma ação.'; detail.append(hint);
  } else if (action.kind === 'media_key') {
    label.textContent = 'Ação multimédia'; input = select(media, action.key, `${labels[index]}: ação multimédia`);
    input.onchange = () => { action.key = Number(input.value); changed(); };
  } else if (action.kind === 'load_profile') {
    label.textContent = 'Perfil';
    const profiles = (status?.files?.profiles || []).map(name => [name, name]);
    if (action.profile && !profiles.some(([name]) => name === action.profile)) profiles.unshift([action.profile, `${action.profile} (indisponível)`]);
    input = select([['', 'Escolher perfil…'], ...profiles], action.profile, `${labels[index]}: perfil`);
    input.onchange = () => { action.profile = input.value; changed(); };
  } else if (action.kind === 'toggle_mute') {
    label.textContent = 'Fader / canal'; input = select(['A','B','C','D'].map(name => { const channel = status?.mixers?.[serial]?.fader_status?.[name]?.channel; const translated = ({Mic:'Microfone',Chat:'Chat',Music:'Música',System:'Sistema',Game:'Jogo'})[channel] || channel || ''; return [name, `Fader ${name}${translated ? ' · ' + translated : ''}`]; }), action.fader, `${labels[index]}: fader`);
    input.onchange = () => { action.fader = input.value; changed(); };
  } else if (action.kind === 'open_program') {
    label.textContent = 'Caminho da aplicação (.exe)'; input = document.createElement('input'); input.type = 'text'; input.value = action.path; input.placeholder = 'C:\\Program Files\\Aplicação\\aplicacao.exe';
    input.setAttribute('aria-label', `${labels[index]}: caminho da aplicação`); input.oninput = () => { action.path = input.value; changed(); };
  } else if (action.kind === 'shortcut') {
    label.textContent = 'Combinação de teclas'; input = document.createElement('button'); input.type = 'button';
    input.textContent = recording === index ? 'Prime o atalho… Esc para cancelar' : (action.keys.length ? describeKeys(action.keys) : 'Gravar atalho');
    input.setAttribute('aria-label', `${labels[index]}: gravar atalho`); input.classList.toggle('recording', recording === index);
    input.onclick = () => { recording = index; render(); controls(); document.getElementById(`value-${index}`).focus(); };
    const hint = document.createElement('p'); hint.textContent = 'Clica para gravar. Usa Ctrl, Alt, Shift ou Windows com uma letra/número, ou uma tecla F1–F24.';
    detail.append(hint);
  }
  if (input) { input.id = `value-${index}`; detail.prepend(label, input); }
  parent.append(detail);
}
function coloursFor(index) {
  const container = document.createElement('div'); container.className = 'colour-controls';
  for (const [property, title] of [['colours','Cor no modo de ações'], ['pressed_colours','Cor ao premir']]) {
    const label = document.createElement('label'); label.textContent = title;
    const group = document.createElement('span'); group.className = 'colour-value';
    const input = document.createElement('input'); input.type = 'color'; input.value = pages[selectedPage][property][index]; input.setAttribute('aria-label', `${index === 5 ? 'Navegação' : labels[index]}: ${title}`);
    const value = document.createElement('output'); value.textContent = input.value.toUpperCase();
    input.oninput = () => { pages[selectedPage][property][index] = input.value; value.textContent = input.value.toUpperCase(); changed(); };
    group.append(input, value); label.append(group); container.append(label);
  }
  return container;
}
function renderPages() {
  $('page-select').replaceChildren(...pages.map((page, index) => option(index, `${index + 1} · ${page.name || 'Sem nome'}`)));
  $('page-select').value = selectedPage; $('page-name').value = pages[selectedPage].name;
  $('navigation-colours').replaceChildren(coloursFor(5));
}
function selectPage(index) {
  recording = null; selectedPage = index; actions = pages[index].actions; renderPages(); render(); controls();
}
function render() {
  $('rows').replaceChildren();
  for (let index = 0; index < 5; index++) {
    const row = document.createElement('div'); row.className = 'binding-row';
    const title = document.createElement('div'); const heading = document.createElement('h3'); heading.textContent = labels[index];
    const normal = document.createElement('span'); normal.className = 'normal'; normal.textContent = index === 4 ? 'Normal: censura' : `Normal: mute ${'ABCD'[index]}`;
    title.append(heading, normal);
    const body = document.createElement('div');
    const type = select(kinds, actions[index].kind, `${labels[index]}: segunda função`);
    type.onchange = () => {
      recording = null;
      actions[index] = ({none:{kind:'none'},media_key:{kind:'media_key',key:179},shortcut:{kind:'shortcut',keys:[]},load_profile:{kind:'load_profile',profile:''},toggle_mute:{kind:'toggle_mute',fader:'ABCD'[Math.min(index,3)]},open_program:{kind:'open_program',path:''}})[type.value];
      changed(); render(); controls(); document.querySelectorAll('.binding-row')[index].querySelector('select').focus();
    };
    body.append(type); detailFor(index, body); body.append(coloursFor(index));
    const test = document.createElement('button'); test.type = 'button'; test.className = 'test-action'; test.textContent = 'Testar ação'; test.setAttribute('aria-label', `Testar ${labels[index]}`); test.onclick = () => testAction(index); body.append(test); row.append(title, body); $('rows').append(row);
  }
}
document.addEventListener('keydown', event => {
  if (recording === null) return;
  event.preventDefault();
  if (event.key === 'Escape') { cancelRecording(); return; }
  if (event.repeat || ['Control','Alt','Shift','Meta'].includes(event.key)) return;
  let key;
  if (/^Key[A-Z]$/.test(event.code)) key = event.code.charCodeAt(3);
  else if (/^Digit[0-9]$/.test(event.code)) key = event.code.charCodeAt(5);
  else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(event.key)) key = 111 + Number(event.key.slice(1));
  if (!key || (key < 112 && !event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey)) {
    message('Usa um modificador com uma letra/número, ou uma tecla F1–F24.', true); return;
  }
  const keys = [];
  if (event.ctrlKey) keys.push(17); if (event.altKey) keys.push(18); if (event.shiftKey) keys.push(16); if (event.metaKey) keys.push(91); keys.push(key);
  actions[recording] = {kind:'shortcut',keys}; recording = null; changed(); render(); controls();
});
async function request(path, options) {
  const response = await fetch(path, {...options, signal: AbortSignal.timeout(5000)});
  if (!response.ok) {
    if (response.status === 404) throw new Error('Esta versão do Utility ainda não inclui ações dos botões. Abre a versão personalizada.');
    throw new Error(await response.text());
  }
  return response.json();
}
function restore() { recording = null; $('enabled').checked = saved.enabled; pages = structuredClone(saved.pages).map(page => ({...page, colours:page.colours || Array(6).fill('#b06cff'), pressed_colours:page.pressed_colours || Array(6).fill('#ffffff')})); dirty = false; selectPage(Math.min(selectedPage, pages.length - 1)); message('Configurações guardadas.'); controls(); }
async function load() {
  available = false; controls(); message('A carregar configurações…');
  try { saved = await request(`/api/button-actions/${encodeURIComponent(serial)}`); available = true; restore(); }
  catch (error) { message(error.message || 'Não foi possível carregar. Volta a abrir esta página.', true); controls(); }
}
async function poll() {
  if (polling) return;
  polling = true;
  try {
    status = await request('/api/get-devices');
    const devices = Object.entries(status.mixers || {}).filter(([,mixer]) => mixer.hardware.device_type === 'Mini');
    if (!serial && devices.length) {
      $('device').replaceChildren(...devices.map(([id]) => option(id, `GoXLR Mini · ${id.slice(-4)}`)));
      serial = devices[0][0]; await load();
    }
    const mixer = status.mixers?.[serial];
    const connected = mixer?.hardware?.device_type === 'Mini';
    available = Boolean(connected && saved);
    const live = connected ? await request(`/api/button-actions/${encodeURIComponent(serial)}`) : null;
    const runtime = live?.runtime;
    if (testing === null && live?.last_result && JSON.stringify(live.last_result) !== lastReport) { lastReport = JSON.stringify(live.last_result); $('action-result').textContent = live.last_result.message; $('action-result').classList.toggle('error', !live.last_result.success); }
    $('layer-state').textContent = !connected ? 'Mini desligado · volta a ligá-lo' : runtime?.active ? `Modo ações · página ${runtime.page + 1}: ${live.pages[runtime.page]?.name || ''}` : saved?.enabled ? 'Modo normal · pressão longa para entrar' : 'Funções originais ativas';
    for (const element of document.querySelectorAll('[data-hardware]')) element.classList.toggle('pressed', Boolean(mixer?.button_down?.[element.dataset.hardware]));
    if (!connected && !devices.length && !serial) { $('device').replaceChildren(option('', 'Nenhum Mini ligado')); message('Liga um GoXLR Mini para configurar as ações.'); }
    controls();
  } catch (error) { available = false; $('layer-state').textContent = 'Sem ligação ao Utility'; message('Não foi possível ligar ao Utility. Confirma que está aberto e volta a carregar a página.', true); controls(); }
  finally { polling = false; }
}
async function testAction(index) {
  if (!available || testing !== null || saving) return;
  testing = index; controls(); $('action-result').textContent = 'A testar…';
  try {
    await request(`/api/button-actions/${encodeURIComponent(serial)}/test`, {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(actions[index])});
    $('action-result').textContent = 'Ação executada. Confirma o resultado no canal ou na aplicação de destino.'; $('action-result').classList.remove('error');
  } catch (error) { $('action-result').textContent = error.message; $('action-result').classList.add('error'); }
  finally { testing = null; controls(); }
}
$('enabled').onchange = changed;
$('page-select').onchange = () => selectPage(Number($('page-select').value));
$('page-name').oninput = () => { pages[selectedPage].name = $('page-name').value; $('page-select').options[selectedPage].textContent = `${selectedPage + 1} · ${pages[selectedPage].name || 'Sem nome'}`; changed(); };
$('add-page').onclick = () => { pages.push(newPage(`Página ${pages.length + 1}`, pages.length)); selectPage(pages.length - 1); changed(); $('page-name').focus(); };
$('duplicate-page').onclick = () => { const copy = structuredClone(pages[selectedPage]); copy.name = `${copy.name} (cópia)`; pages.push(copy); selectPage(pages.length - 1); changed(); };
$('remove-page').onclick = () => { if (pages.length < 2) return; pages.splice(selectedPage, 1); selectPage(Math.min(selectedPage, pages.length - 1)); changed(); };
$('device').onchange = async () => { serial = $('device').value; saved = undefined; dirty = false; await load(); };
$('reset').onclick = restore;
$('actions-form').onsubmit = async event => {
  event.preventDefault(); if (!available || saving || testing !== null || recording !== null) return;
  const layer = {enabled:$('enabled').checked, pages:structuredClone(pages)};
  if (layer.pages.some(page => !page.name.trim())) { message('Dá um nome a todas as páginas antes de guardar.', true); return; }
  if (layer.pages.flatMap(page => page.actions).some(action => (action.kind === 'shortcut' && !action.keys.length) || (action.kind === 'load_profile' && !action.profile) || (action.kind === 'open_program' && !action.path.trim()))) {
    message('Completa o atalho, perfil ou caminho das ações escolhidas antes de guardar.', true); return;
  }
  saving = true; controls(); message('A guardar…');
  try { await request(`/api/button-actions/${encodeURIComponent(serial)}`, {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(layer)}); saved = layer; restore(); }
  catch (error) { message(error.message || 'Não foi possível guardar. Tenta novamente.', true); }
  finally { saving = false; controls(); }
};
window.addEventListener('beforeunload', event => { if (dirty) { event.preventDefault(); event.returnValue = ''; } });
renderPages(); render(); controls(); poll(); setInterval(poll, 1200);
