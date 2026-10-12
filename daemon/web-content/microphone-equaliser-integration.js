const miniBands = [
  ['Equalizer90Hz', '90 Hz'],
  ['Equalizer250Hz', '250 Hz'],
  ['Equalizer500Hz', '500 Hz'],
  ['Equalizer1KHz', '1 kHz'],
  ['Equalizer3KHz', '3 kHz'],
  ['Equalizer8KHz', '8 kHz'],
];
const fullBands = [
  ['Equalizer31Hz', '31 Hz'],
  ['Equalizer63Hz', '63 Hz'],
  ['Equalizer125Hz', '125 Hz'],
  ['Equalizer250Hz', '250 Hz'],
  ['Equalizer500Hz', '500 Hz'],
  ['Equalizer1KHz', '1 kHz'],
  ['Equalizer2KHz', '2 kHz'],
  ['Equalizer4KHz', '4 kHz'],
  ['Equalizer8KHz', '8 kHz'],
  ['Equalizer16KHz', '16 kHz'],
];
const svgNS = 'http://www.w3.org/2000/svg';
let serial = '';
let kind = '';
let bands = [];
let current = null;
let snapshots = { A: null, B: null };
let comparing = false;
let writing = false;
let loading = false;
let connected = false;
let panel;

const makeSvg = (name, attributes = {}) => {
  const element = document.createElementNS(svgNS, name);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, value);
  return element;
};

function setMessage(message, error = false) {
  const output = panel?.querySelector('[data-message]');
  if (!output) return;
  output.textContent = message;
  output.classList.toggle('error', error);
}

function snapshotKey(slot) {
  return `goxlr-${kind === 'Mini' ? 'mini-' : ''}eq:${serial}:${slot}`;
}

function readSnapshots() {
  snapshots = { A: null, B: null };
  for (const slot of ['A', 'B']) {
    try {
      const value = JSON.parse(localStorage.getItem(snapshotKey(slot)) || 'null');
      if (value && bands.every(([id]) => Number.isFinite(Number(value.gain?.[id])) && Number.isFinite(Number(value.frequency?.[id])))) {
        snapshots[slot] = value;
      }
    } catch {
      snapshots[slot] = null;
    }
  }
  updateButtons();
}

function updateButtons() {
  if (!panel) return;
  for (const button of panel.querySelectorAll('button[data-action]')) {
    const action = button.dataset.action;
    button.disabled = !connected || writing || (action === 'compare' && !snapshots.A && !snapshots.B) || (action === 'apply-a' && !snapshots.A) || (action === 'apply-b' && !snapshots.B);
  }
  const compare = panel.querySelector('[data-action="compare"]');
  if (compare) compare.textContent = comparing ? 'Ocultar A/B' : 'Comparar A/B';
}

function graphX(frequency) {
  return 48 + (Math.log10(frequency) - Math.log10(20)) / 3 * 730;
}

function graphY(gain) {
  return 18 + (12 - Math.max(-12, Math.min(12, gain))) / 24 * 202;
}

function drawCurve(gains, frequencies) {
  const points = bands.map(([id]) => ({
    x: graphX(Number(frequencies[id])),
    y: graphY(Number(gains[id])),
  }));
  return points.map((point, index) => `${index ? 'L' : 'M'} ${point.x.toFixed(1)} ${point.y.toFixed(1)}`).join(' ');
}

function drawGraph() {
  if (!panel) return;
  const grid = panel.querySelector('[data-grid]');
  const paths = panel.querySelector('[data-curves]');
  const labels = panel.querySelector('[data-axis-labels]');
  grid.replaceChildren();
  labels.replaceChildren();
  paths.replaceChildren();
  for (const gain of [-9, -6, -3, 0, 3, 6, 9]) {
    const y = graphY(gain);
    grid.append(makeSvg('line', { x1: 48, x2: 778, y1: y, y2: y, class: gain === 0 ? 'zero' : '' }));
    const label = makeSvg('text', { x: 42, y: y + 4, 'text-anchor': 'end' });
    label.textContent = `${gain} dB`;
    grid.append(label);
  }
  for (const frequency of [20, 50, 100, 250, 500, 1000, 3000, 8000, 20000]) {
    const x = graphX(frequency);
    grid.append(makeSvg('line', { x1: x, x2: x, y1: 18, y2: 220, class: 'vertical' }));
    const label = makeSvg('text', { x, y: 250, 'text-anchor': 'middle' });
    label.textContent = frequency >= 1000 ? `${frequency / 1000}k` : String(frequency);
    labels.append(label);
  }
  if (!current) return;

  for (const slot of ['A', 'B']) {
    if (!comparing || !snapshots[slot]) continue;
    paths.append(makeSvg('path', { d: drawCurve(snapshots[slot].gain, snapshots[slot].frequency), class: `saved-${slot.toLowerCase()}` }));
  }
  paths.append(makeSvg('path', { d: drawCurve(current.gain, current.frequency), class: 'current' }));
  const points = bands.map(([id]) => `${id}: ${Number(current.gain[id]) > 0 ? '+' : ''}${current.gain[id]} dB`);
  panel.querySelector('[data-readout]').textContent = points.join(' · ');
}

function mountPanel() {
  const checkbox = document.getElementById('eq_fine');
  const container = checkbox?.closest('.expando')?.querySelector('.container');
  if (!container) return;
  if (container.querySelector('#goxlr-mic-eq-tools')) {
    panel = container.querySelector('#goxlr-mic-eq-tools');
    return;
  }

  panel = document.createElement('section');
  panel.id = 'goxlr-mic-eq-tools';
  panel.className = 'goxlr-mic-eq-tools';
  panel.setAttribute('aria-labelledby', 'goxlr-mic-eq-title');
  panel.innerHTML = `
    <div class="goxlr-mic-eq-heading">
      <div><h3 id="goxlr-mic-eq-title">Curva e comparação A/B</h3><p>O gráfico acompanha as bandas ajustadas nos controlos acima.</p></div>
      <output data-readout aria-live="polite"></output>
    </div>
    <svg class="goxlr-mic-eq-chart" viewBox="0 0 800 270" role="img" aria-label="Curva indicativa do equalizador do microfone">
      <g data-grid></g><g data-curves></g><g data-axis-labels></g>
    </svg>
    <div class="goxlr-mic-eq-footer">
      <div class="goxlr-mic-eq-legend"><span><i class="current"></i>Atual</span><span><i class="saved-a"></i>A</span><span><i class="saved-b"></i>B</span></div>
      <div class="goxlr-mic-eq-controls">
        <button type="button" data-action="save-a">Guardar A</button>
        <button type="button" data-action="save-b">Guardar B</button>
        <button type="button" data-action="compare">Comparar A/B</button>
        <button type="button" data-action="apply-a">Aplicar A</button>
        <button type="button" data-action="apply-b">Aplicar B</button>
      </div>
    </div>
    <p class="goxlr-mic-eq-message" data-message role="status" aria-live="polite">A ligar ao equalizador…</p>
    <p class="goxlr-mic-eq-note">A curva é indicativa, não mede o som do microfone. As configurações A/B ficam guardadas neste computador por dispositivo.</p>`;
  container.append(panel);
  panel.addEventListener('click', event => {
    const action = event.target.closest('button[data-action]')?.dataset.action;
    if (action === 'save-a' || action === 'save-b') save(action.slice(-1).toUpperCase());
    if (action === 'compare') {
      comparing = !comparing;
      updateButtons();
      drawGraph();
    }
    if (action === 'apply-a') apply('A');
    if (action === 'apply-b') apply('B');
  });
  updateButtons();
  drawGraph();
  refresh();
}

function save(slot) {
  if (!current) return;
  snapshots[slot] = structuredClone(current);
  localStorage.setItem(snapshotKey(slot), JSON.stringify(snapshots[slot]));
  setMessage(`Configuração ${slot} guardada neste computador.`);
  updateButtons();
  drawGraph();
}

async function sendCommand(command, deviceSerial = serial) {
  const response = await fetch('/api/command', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ Command: [deviceSerial, command] }),
    signal: AbortSignal.timeout(8000),
  });
  const data = await response.json();
  if (!response.ok || data.Error) throw new Error(data.Error || 'O Utility recusou o ajuste.');
}

async function apply(slot) {
  const saved = snapshots[slot];
  if (!saved || writing) return;
  const targetSerial = serial;
  const targetKind = kind;
  const targetBands = [...bands];
  writing = true;
  updateButtons();
  setMessage(`A aplicar a configuração ${slot}…`);
  try {
    for (const [id] of targetBands) {
      const frequencyCommand = targetKind === 'Mini' ? 'SetEqMiniFreq' : 'SetEqFreq';
      const gainCommand = targetKind === 'Mini' ? 'SetEqMiniGain' : 'SetEqGain';
      await sendCommand({ [frequencyCommand]: [id, Number(saved.frequency[id])] }, targetSerial);
      await sendCommand({ [gainCommand]: [id, Number(saved.gain[id])] }, targetSerial);
    }
    if (serial === targetSerial) current = structuredClone(saved);
    setMessage(`Configuração ${slot} aplicada.`);
    drawGraph();
  } catch (error) {
    setMessage(error.message || `Não foi possível aplicar ${slot}.`, true);
  } finally {
    writing = false;
    updateButtons();
    refresh();
  }
}

function selectDeviceFromClick(event) {
  const button = event.target.closest('.buttonList .button');
  if (!button) return;
  const label = button.querySelector('.left_side')?.textContent || button.textContent;
  const match = label.match(/^\s*\[([^\]]+)\]/);
  if (!match || serial === match[1]) return;
  serial = match[1];
  kind = '';
  current = null;
  comparing = false;
  connected = false;
  updateButtons();
  refresh();
}

async function refresh() {
  if (loading || !panel?.isConnected || writing) return;
  loading = true;
  try {
    const response = await fetch('/api/get-devices', { signal: AbortSignal.timeout(6000) });
    if (!response.ok) throw new Error('Não foi possível consultar o equalizador.');
    const data = await response.json();
    const mixers = data.mixers || {};
    if (serial && !mixers[serial]) serial = '';
    if (!serial && Object.keys(mixers).length === 1) serial = Object.keys(mixers)[0];
    if (!serial || !mixers[serial]) {
      connected = false;
      setMessage('Seleciona um GoXLR na aplicação para ver a curva.');
      updateButtons();
      return;
    }

    const mixer = mixers[serial];
    const nextKind = mixer.hardware?.device_type || '';
    const nextBands = nextKind === 'Mini' ? miniBands : fullBands;
    const equalizer = nextKind === 'Mini' ? mixer.mic_status?.equaliser_mini : mixer.mic_status?.equaliser;
    if (!equalizer?.gain || !equalizer?.frequency) {
      connected = false;
      setMessage('Não foi possível ler as bandas do equalizador.', true);
      updateButtons();
      return;
    }

    const changedDevice = kind !== nextKind;
    kind = nextKind;
    bands = nextBands;
    current = { gain: { ...equalizer.gain }, frequency: { ...equalizer.frequency } };
    connected = true;
    if (changedDevice) {
      comparing = false;
      readSnapshots();
      setMessage('Curva ligada ao equalizador do separador Mic.');
    }
    updateButtons();
    drawGraph();
  } catch (error) {
    connected = false;
    setMessage(error.message || 'Não foi possível ligar ao Utility.', true);
    updateButtons();
  } finally {
    loading = false;
  }
}

document.addEventListener('click', selectDeviceFromClick, true);
new MutationObserver(mountPanel).observe(document.body, { childList: true, subtree: true });
setInterval(() => {
  mountPanel();
  refresh();
}, 1200);
