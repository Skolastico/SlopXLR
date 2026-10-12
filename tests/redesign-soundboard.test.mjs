import {test} from 'node:test';
import assert from 'node:assert/strict';
import {folderEntries,sampleUrl,audioLibrary,library} from '../daemon/web-content/redesign/soundboard.mjs';
import {sampler} from '../daemon/web-content/redesign/features.mjs';
import {newAction} from '../daemon/web-content/redesign/model.mjs';
test('library folders preserve full paths and escaped names',()=>{
 const files={'folder\\sound[1].wav':{name:'Sound'},'other/sound[1].wav':{name:'Other'},'root.wav':{name:'<Root>'}};
 assert.deepEqual(folderEntries(files).folders,['folder','other']);
 assert.equal(folderEntries(files,'folder').samples[0].file,'folder\\sound[1].wav');
 assert.equal(sampleUrl('folder\\áudio #1.wav'),'/files/samples/folder/%C3%A1udio%20%231.wav');
 library.folder='';library.selected=null;library.clip=null;
 assert(audioLibrary({samples:files}).includes('&lt;Root&gt;'));
});
test('Mini exposes virtual sampler playback without hardware recording controls',()=>{
 const m={hardware:{device_type:'Mini'},sampler:{active_bank:'A',banks:{A:{TopLeft:{samples:[{name:'root.wav',start_pct:0,stop_pct:100}],order:'Sequential',function:'PlayNext'}}}}};
 const html=sampler(m,{samples:{'root.wav':{name:'Root'}}});
 assert(html.includes('canal Sistema'));assert(html.includes('Ver onda e cortar'));assert(html.includes('PlayNextSample'));assert(!html.includes('SetSamplerPreBufferDuration'));assert(!html.includes('SetSamplerFunction'));
 assert.deepEqual(newAction('play_sample'),{kind:'play_sample',bank:'A',button:'TopLeft'});
 assert.deepEqual(newAction('stop_sample'),{kind:'stop_sample',bank:'A',button:'TopLeft'});
});
