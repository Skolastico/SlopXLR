const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const source = fs.readFileSync(path.join(__dirname, '../daemon/web-content/application-controls.js'), 'utf8');
const elements = {};
const context = vm.createContext({document: {getElementById: id => elements[id] ||= {}}, structuredClone, setInterval() {}});
// Exercise the actual positioning logic without starting device polling.
vm.runInContext(source.replace('poll(); setInterval(poll, 1500);', ''), context);
vm.runInContext(`
  changed = () => {};
  draft = [
    {name:'Spotify', channel:'Music', slot:0},
    {name:'Browser', channel:'Music', slot:2},
    {name:'Game', channel:'Game', slot:2}
  ];
  setSlot(draft[0], 2);
`, context);
assert.equal(vm.runInContext('draft[0].slot', context), 2, 'Chosen fader C is preserved');
assert.equal(vm.runInContext('draft[1].slot', context), 0, 'Occupied faders swap');
assert.equal(vm.runInContext('draft[2].slot', context), 2, 'Other channels remain independent');
assert.equal(vm.runInContext("nextSlot('Music')", context), 1);
vm.runInContext('setSlot(draft[0], 7)', context);
assert.equal(vm.runInContext('draft[0].slot', context), 7, 'Page 2, fader D remains at slot 7');
console.log('Application slots: chosen faders, swaps, page gaps and channel isolation passed.');
