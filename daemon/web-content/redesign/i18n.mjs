import english from './locales/en.json' with {type:'json'};
export const languageKey='goxlr-personalizado-language';
export function resolveLanguage(saved,system='en'){return ['pt-PT','en'].includes(saved)?saved:/^pt(?:[-_]|$)/i.test(system)?'pt-PT':'en';}
let language='pt-PT';
if(typeof navigator!=='undefined'){let saved;try{saved=localStorage.getItem(languageKey);}catch{}language=resolveLanguage(saved,navigator.language);}
export const getLanguage=()=>language;
export function setLanguage(value){if(!['pt-PT','en'].includes(value))throw new Error('Unsupported language');language=value;try{localStorage.setItem(languageKey,value);}catch{}if(typeof document!=='undefined')document.documentElement.lang=value;}
function text(value){const trimmed=value.trim();return Object.hasOwn(english,trimmed)?value.replace(trimmed,english[trimmed]):value;}
// Translate only authored literal pieces. Interpolated names, paths and device data stay untouched.
function literal(value){if(language!=='en')return value;if(Object.hasOwn(english,value.trim()))return text(value);
return value.replace(/((?:aria-label|title|placeholder)=")([^"]*)(?="|$)/g,(_,prefix,label)=>prefix+text(label))
.replace(/(^|>)([^<>]*)(?=<|$)/g,(_,prefix,label)=>prefix+text(label));}
export function t(value,...values){if(Array.isArray(value)&&Object.hasOwn(value,'raw'))return value.map((part,i)=>literal(part)+(i<values.length?values[i]:'')).join('');return literal(String(value));}
export function translateDocument(root=document){document.documentElement.lang=language;const walk=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);let node;while(node=walk.nextNode()){if(!node.parentElement?.closest('script,style'))node.textContent=literal(node.textContent);}for(const node of root.querySelectorAll('[aria-label],[title],[placeholder]'))for(const attr of ['aria-label','title','placeholder'])if(node.hasAttribute(attr))node.setAttribute(attr,literal(node.getAttribute(attr)));}
