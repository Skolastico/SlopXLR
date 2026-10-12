import {t as __tr} from './i18n.mjs';
export const faders=[__tr('A'),__tr('B'),__tr('C'),__tr('D')];
export const sampleSlots=[[__tr('TopLeft'),__tr('Áudio 1 · Superior esquerdo')],[__tr('TopRight'),__tr('Áudio 2 · Superior direito')],[__tr('BottomLeft'),__tr('Áudio 3 · Inferior esquerdo')],[__tr('BottomRight'),__tr('Áudio 4 · Inferior direito')]];
export const sampleSlotName=button=>sampleSlots.find(([id])=>id===button)?.[1]||__tr('Áudio por escolher');
export const inputs=[__tr('Mic'),__tr('Chat'),__tr('Music'),__tr('System'),__tr('Game'),__tr('Console'),__tr('LineIn'),__tr('Sample')];
export const appChannels=[__tr('Music'),__tr('Chat'),__tr('Game'),__tr('System'),__tr('Sample')];
export const labels={Mic:__tr('Microfone'),Chat:__tr('Chat'),Music:__tr('Música'),System:__tr('Sistema'),Game:__tr('Jogo'),Console:__tr('Consola'),LineIn:__tr('Line In'),Sample:__tr('Samples'),Headphones:__tr('Auscultadores'),MicMonitor:__tr('Monitor do microfone'),LineOut:__tr('Line Out'),BroadcastMix:__tr('Stream Mix'),ChatMic:__tr('Chat Mic'),Sampler:__tr('Sampler'),Mini:__tr('GoXLR Mini'),Full:__tr('GoXLR')};
export const actionKinds={none:__tr('Sem ação'),open_program:__tr('Abrir aplicação'),media_key:__tr('Controlar multimédia'),shortcut:__tr('Atalho de teclado'),load_profile:__tr('Carregar perfil'),toggle_mute:__tr('Alternar mute de um fader'),play_sample:__tr('Reproduzir áudio do soundboard'),stop_sample:__tr('Parar áudio do soundboard')};
export const media=[[179,__tr('Play / pause')],[176,__tr('Faixa seguinte')],[177,__tr('Faixa anterior')],[178,__tr('Parar')],[173,__tr('Silenciar áudio do Windows')],[174,__tr('Diminuir volume do Windows')],[175,__tr('Aumentar volume do Windows')]];
export const esc=value=>String(value??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export const dirty=(a,b)=>JSON.stringify(a)!==JSON.stringify(b);
export const isMini=mixer=>mixer?.hardware?.device_type===__tr('Mini');
export const pct=value=>Math.round(Number(value)*100/255);
export const volume=value=>Math.round(Math.max(0,Math.min(100,Number(value)))*255/100);
export const pageCount=(apps,channel)=>Math.max(1,...apps.filter(a=>a.channel===channel).map(a=>Math.floor((a.slot??0)/4)+1));
export function freeSlot(apps,channel,exclude=-1){for(let slot=0;slot<400;slot++)if(!apps.some((a,i)=>i!==exclude&&a.channel===channel&&a.slot===slot))return slot;throw new Error(__tr('As 100 páginas deste canal estão ocupadas.'));}
export function moveBinding(apps,index,channel,slot){
  if(!Number.isInteger(slot)||slot<0||slot>=400)throw new Error(__tr('Escolhe uma página entre 1 e 100.'));
  const current=apps[index];if(!current)throw new Error(__tr('A associação já não existe.'));
  const other=apps.find((a,i)=>i!==index&&a.channel===channel&&a.slot===slot);
  if(other)Object.assign(other,{channel:current.channel,slot:current.slot});
  Object.assign(current,{channel,slot});return Boolean(other);
}
export function newAction(kind){return ({none:{kind},open_program:{kind,path:''},media_key:{kind,key:179},shortcut:{kind,keys:[]},load_profile:{kind,profile:''},toggle_mute:{kind,fader:__tr('A')},play_sample:{kind,bank:__tr('A'),button:__tr('TopLeft')},stop_sample:{kind,bank:__tr('A'),button:__tr('TopLeft')}})[kind];}
export function newPage(name){return {name,actions:Array.from({length:5},()=>newAction('none')),colours:Array(6).fill('#009dff'),pressed_colours:Array(6).fill('#ffffff')};}
export function describe(action){if(!action)return __tr('Sem ação');switch(action.kind){case 'open_program':return action.path?__tr`Abrir ${action.path.split(/[\\/]/).pop()}`:__tr('Escolher programa');case 'media_key':return media.find(([id])=>id===action.key)?.[1]||__tr('Comando multimédia');case 'load_profile':return __tr`Perfil ${action.profile||__tr('por escolher')}`;case 'play_sample':return __tr`Reproduzir · Banco ${action.bank} / ${sampleSlotName(action.button)}`;case 'stop_sample':return __tr`Parar · Banco ${action.bank} / ${sampleSlotName(action.button)}`;case 'toggle_mute':return __tr`Mute do fader ${action.fader}`;case 'shortcut':return action.keys?.length?describeKeys(action.keys):__tr('Gravar atalho');default:return __tr('Sem ação');}}
export function describeKeys(keys){return keys.map(k=>({16:__tr('Shift'),17:__tr('Ctrl'),18:__tr('Alt'),91:__tr('Windows'),13:__tr('Enter'),32:__tr('Espaço'),27:__tr('Esc')})[k]||(k>=112&&k<=135?__tr`F${k-111}`:String.fromCharCode(k))).join(' + ');}
export function validateLayer(layer,profiles=[]){
  if(!layer.pages.length||layer.pages.some(p=>!p.name.trim()))throw new Error(__tr('Dá um nome a todas as páginas.'));
  for(const p of layer.pages)for(const a of p.actions){if(a.kind==='open_program'&&!/\.exe$/i.test(a.path.trim()))throw new Error(__tr`Completa o caminho .exe em ${p.name}.`);if(a.kind==='shortcut'&&!a.keys.length)throw new Error(__tr`Grava o atalho em ${p.name}.`);if(a.kind==='load_profile'&&!profiles.includes(a.profile))throw new Error(__tr`Escolhe um perfil disponível em ${p.name}.`);}
}
