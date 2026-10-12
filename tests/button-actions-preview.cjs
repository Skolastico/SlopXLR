// Development preview only. This server never talks to the installed Utility or USB hardware.
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
let layer = {enabled:false,pages:[{name:'Página 1',actions:Array.from({length:5},()=>({kind:'none'}))}]};
const status = {mixers:{'PREVIEW-MINI':{hardware:{device_type:'Mini'},button_down:{}}},files:{profiles:['Default','Default - Red','Sleep']}};
http.createServer((req,res)=>{
  const url = new URL(req.url, 'http://127.0.0.1');
  if(url.pathname === '/api/get-devices'){res.setHeader('Content-Type','application/json');return res.end(JSON.stringify(status));}
  if(url.pathname === '/api/button-actions/PREVIEW-MINI'){
    res.setHeader('Content-Type','application/json');
    if(req.method === 'POST'){
      let body='';req.on('data',chunk=>body+=chunk);req.on('end',()=>{try{layer=JSON.parse(body);res.end('{"saved":true}');}catch{res.writeHead(400);res.end('JSON inválido');}});return;
    }
    return res.end(JSON.stringify({...layer,runtime:{active:false,page:0}}));
  }
  const name = url.pathname === '/' ? 'button-actions.html' : url.pathname.slice(1);
  if(!['button-actions.html','button-actions.js','button-actions.css'].includes(name)){res.writeHead(404);return res.end();}
  let contents=fs.readFileSync(path.join(__dirname,"../daemon/web-content",name));
  if(name.endsWith('.html')) contents=contents.toString().replace('<span>Utility</span>','<span>Utility · Pré-visualização</span>');
  res.setHeader('Content-Type',name.endsWith('.css')?'text/css':name.endsWith('.js')?'text/javascript':'text/html; charset=utf-8');res.end(contents);
}).listen(Number(process.env.GOXLR_PREVIEW_PORT || 14565),'127.0.0.1',()=>console.log('Simulated Mini preview started'));

