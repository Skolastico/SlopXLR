import {t as __tr,getLanguage,translateDocument} from './redesign/i18n.mjs';
translateDocument();
const $ = id => document.getElementById(id);
let loading = false;

function addEvent(list, event) {
  const item = document.createElement('li');
  const time = document.createElement('time');
  time.textContent = event.time || '—';
  const detail = document.createElement('p');
  detail.textContent = event.message;
  if (event.kind === 'issue') detail.classList.add('error');
  item.append(time, detail);
  list.append(item);
}

async function refresh() {
  if (loading) return;
  loading = true;
  $('refresh').disabled = true;
  try {
    const response = await fetch('/api/diagnostics', {signal:AbortSignal.timeout(5000)});
    if (!response.ok) throw new Error(__tr('Não foi possível consultar o diagnóstico.'));
    const report = await response.json();
    $('connection').textContent = report.mini_connected === true
      ? __tr('Ligado')
      : report.mini_connected === false ? __tr('Não detetado') : __tr('Sem resposta do Utility');
    $('connection').classList.toggle('error', report.mini_connected !== true);
    $('last-issue').textContent = report.last_usb_issue
      ? `${report.last_usb_issue.time} · ${report.last_usb_issue.message}`
      : report.log_available ? __tr('Sem falhas USB registadas.') : __tr('Registo ainda indisponível.');
    $('last-issue').classList.toggle('error', Boolean(report.last_usb_issue));
    $('recovery').textContent = report.recovery
      ? `${report.recovery.message}${report.recovery_seconds === null ? '' : __tr` · cerca de ${report.recovery_seconds} s`}`
      : report.last_usb_issue ? __tr('Sem recuperação registada depois desta falha.') : __tr('Sem dados de recuperação.');
    const events = $('events');
    events.replaceChildren();
    for (const event of report.events) addEvent(events, event);
    if (!report.events.length) {
      const empty = document.createElement('li');
      empty.className = 'subtle';
      empty.textContent = report.log_available ? __tr('Ainda não há eventos USB para mostrar.') : __tr('O registo atual ainda não está disponível.');
      events.append(empty);
    }
    $('export').hidden = !report.log_available;
    $('feedback').textContent = __tr`Atualizado às ${new Intl.DateTimeFormat(getLanguage(), {hour:'2-digit', minute:'2-digit', second:'2-digit'}).format(new Date())}.`;
    $('feedback').classList.remove('error');
  } catch (error) {
    $('feedback').textContent = error.message || __tr('Não foi possível consultar o diagnóstico.');
    $('feedback').classList.add('error');
  } finally {
    loading = false;
    $('refresh').disabled = false;
  }
}

$('refresh').addEventListener('click', refresh);
refresh();
setInterval(refresh, 5000);
