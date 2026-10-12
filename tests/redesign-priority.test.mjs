import {test} from 'node:test';
import assert from 'node:assert/strict';
import {eqPath,eqDefaults,eqGroups,lifecycleEditor,readLifecycle} from '../daemon/web-content/redesign/priority.mjs';
import {lightingDetails} from '../daemon/web-content/redesign/features.mjs';
test('global colour uses the aggregate device command with a scalar colour',()=>{
 const html=lightingDetails({hardware:{device_type:'Mini'},lighting:{simple:{Global:{colour_one:'123456'}}}});
 assert(html.includes('SetGlobalColour'));assert(!html.includes('SetSimpleColour'));assert(html.includes('&quot;position&quot;:0'));
});
test('EQ frequencies determine logarithmic position and defaults cover both models',()=>{
 const data={gain:{a:0,b:2},frequency:{a:100,b:1000}},before=eqPath(data,['a','b']);data.frequency.a=200;assert.notEqual(eqPath(data,['a','b']),before);
 assert.deepEqual(eqDefaults(['Equalizer90Hz','Equalizer1KHz','Equalizer16KHz']),{Equalizer90Hz:90,Equalizer1KHz:1000,Equalizer16KHz:16000});
 assert.deepEqual(eqGroups([1,2,3,4,5,6]),[[1,2],[3,4],[5,6]]);
});
test('visual lifecycle editing preserves custom commands and profile persistence',()=>{
 const custom={SetVolume:['Music',80]},previous=[{LoadProfile:['Default',false]},custom,{SaveMicProfile:[]}];
 const form={elements:{'kind-0':{value:'LoadProfile'},'profile-0':{value:'Sleep'},'persist-0':{checked:true},'kind-1':{value:'SetVolume'},'kind-2':{value:'SaveMicProfile'}}};
 assert.deepEqual(readLifecycle(form,previous),[{LoadProfile:['Sleep',true]},custom,{SaveMicProfile:[]}]);
 const html=lifecycleEditor('sleep',previous,{profiles:['Default','Sleep']});assert(html.includes('Comando personalizado conservado'));assert(html.includes('Subir'));assert(!html.includes('<textarea'));
});

import {reverbDecayValues,reverbDecayIndex} from '../daemon/web-content/redesign/priority.mjs';
import {displayMode,micDetails,micProcessing,effects} from '../daemon/web-content/redesign/features.mjs';
import {device} from './redesign-migration.test.mjs';
test('microphone views use device preferences and keep advanced processing available',()=>{const m=device('Mini');assert.equal(displayMode(m,'gate'),'Advanced');assert(micDetails(m).includes('freq-Equalizer90Hz'));m.settings.display.equaliser_fine='Simple';assert(!micDetails(m).includes('freq-Equalizer90Hz'));assert(micProcessing(m,'gate').includes('SetGateAttack'));assert(micProcessing(m,'compressor').includes('SetCompressorRatio'));});
test('lighting groups preserve the source colours and distinguish hardware groups',()=>{const m=device();m.lighting.buttons.EffectSelect1={colours:{colour_one:'112233',colour_two:'445566'},off_style:'Colour2'};m.lighting.buttons.EffectRobot={colours:{colour_one:'778899',colour_two:'000000'},off_style:'Dimmed'};const html=lightingDetails(m);assert(html.includes('SetAllFaderColours'));assert(html.includes('SetAllFaderDisplayStyle'));assert(html.includes('EffectSelector'));assert(html.includes('EffectTypes'));assert(html.includes('112233'));assert(html.includes('SetButtonGroupOffStyle'));});
test('reverb slider values match representable decay rather than a one millisecond scale',()=>{assert.equal(reverbDecayValues[0],10);assert.equal(reverbDecayValues.at(-1),20000);for(const v of reverbDecayValues)assert.equal(v%(v<=1000?10:100),0);assert.equal(reverbDecayValues[reverbDecayIndex(1199)],1100);assert(effects(device()).includes('data-reverb-decay'));});
