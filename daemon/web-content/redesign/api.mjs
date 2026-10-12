import {t as __tr} from './i18n.mjs';
export async function request(path, {method=__tr('GET'), body, signal}={}) {
  let response;try{response=await fetch(path,{method,headers:body===undefined?{}:{'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:signal||AbortSignal.timeout(8000)});}catch(error){throw new Error(error.name===__tr('TimeoutError')?__tr('O serviço GoXLR demorou demasiado a responder. Verifica o estado antes de repetir a operação.'):error.name===__tr('AbortError')?__tr('O pedido ao serviço GoXLR foi interrompido.'):__tr('Não foi possível contactar o serviço GoXLR. Verifica se o GoXLR Personalizado está aberto.'),{cause:error});}
  const text=await response.text();
  let data;try{data=text?JSON.parse(text):null;}catch{data=text;}
  if(!response.ok)throw new Error(typeof data==='string'?data:__tr`Pedido recusado (${response.status}).`);
  if(data&&typeof data==='object'&&Object.hasOwn(data,__tr('Error')))throw new Error(data.Error);
  return data;
}
export const command=(serial,value)=>request('/api/command',{method:__tr('POST'),body:{Command:[serial,value]}});
export const devicePath=(area,serial)=>`/api/${area}/${encodeURIComponent(serial)}`;
