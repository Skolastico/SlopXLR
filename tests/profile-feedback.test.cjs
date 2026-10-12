const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'daemon/web-content/index.html'), 'utf8');
const bundle = fs.readFileSync(path.join(root, 'daemon/web-content/assets/index-BJM830Pw.js'), 'utf8');
const elements = { 'profile-error': { hidden: true }, 'profile-error-message': { textContent: '' } };
const context = vm.createContext({
  window: {}, document: { getElementById: id => elements[id] },
  c: { getActiveSerial: () => 'TEST', setAccessibilityNotification: () => notifications++ },
  console: { log() {} },
});
let notifications = 0;
vm.runInContext(html.match(/window\.showProfileError = function[\s\S]*?\n      };/)[0], context);
vm.runInContext(bundle.slice(bundle.indexOf('function Vt('), bundle.indexOf('function Mp(')), context);
const loader = bundle.match(/loadProfile:function\(e\)\{.*?(?=,loadProfileColours:)/)[0];
const manager = vm.runInContext('({' + loader + '})', context);
manager.$t = () => 'Profile loaded';

(async () => {
  context.Mp = () => Promise.reject('USB <disconnected>');
  await assert.rejects(context.Vt('TEST', { LoadProfile: ['Next', true] }));
  assert.equal(elements['profile-error'].hidden, false);
  assert.match(elements['profile-error-message'].textContent, /USB <disconnected>/);
  assert.match(html, /id="profile-error-message" role="alert"/);
  manager.loadProfile('Next');
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(notifications, 0, 'A failed load must not announce success');
  elements['profile-error'].hidden = true;
  await assert.rejects(context.Vt('TEST', { SetVolume: ['Music', 10] }));
  assert.equal(elements['profile-error'].hidden, true, 'Unrelated commands keep their existing UI');
  let complete;
  context.Mp = () => new Promise(resolve => { complete = resolve; });
  manager.loadProfile('Next');
  assert.equal(notifications, 0, 'A pending load must not announce success');
  complete();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(notifications, 1);
  console.log('Profile feedback: errors visible; success announced only after completion.');
})().catch(error => { console.error(error); process.exitCode = 1; });
