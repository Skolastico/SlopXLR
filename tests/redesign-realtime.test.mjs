import {test} from 'node:test';
import assert from 'node:assert/strict';
import {watchStatus} from '../daemon/web-content/redesign/realtime.mjs';

test('status and patch notifications refresh immediately, reconnect and stop without late callbacks',t=>{
 t.mock.timers.enable({apis:['setTimeout']});
 const sockets=[],states=[];let updates=0;
 class Socket{constructor(url){this.url=url;sockets.push(this);}send(value){this.sent=JSON.parse(value);}close(){this.onclose?.();}}
 const feed=watchStatus('ws://localhost/api/websocket',()=>updates++,state=>states.push(state),Socket);
 const first=sockets[0];first.onopen();assert.deepEqual(first.sent,{id:1,data:'GetStatus'});
 first.onmessage({data:JSON.stringify({id:1,data:{Status:{mixers:{}}}})});
 first.onmessage({data:JSON.stringify({data:{Patch:[]}})});
 assert.equal(updates,2);assert(feed.connected);assert.deepEqual(states,[true]);
 first.onerror();assert(!feed.connected);assert.deepEqual(states,[true,false]);
 t.mock.timers.tick(1500);assert.equal(sockets.length,2);
 sockets[1].onmessage({data:JSON.stringify({data:{Status:{}}})});assert(feed.connected);
 feed.stop();sockets[1].onmessage({data:JSON.stringify({data:{Patch:[]}})});
 t.mock.timers.tick(3000);assert.equal(sockets.length,2);assert.equal(updates,3);assert(!feed.connected);
});
