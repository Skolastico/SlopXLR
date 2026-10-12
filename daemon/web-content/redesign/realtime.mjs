// Notifications reuse the existing HTTP readers; drafts never receive JSON patches.
export function watchStatus(url,onChange,onConnection,Socket=globalThis.WebSocket){
 let socket,retry,closed=false,connected=false;
 const state=value=>{if(connected!==value){connected=value;onConnection(value);}};
 function connect(){
  if(closed||!Socket)return;
  try{socket=new Socket(url);}catch{retry=setTimeout(connect,1500);return;}
  socket.onopen=()=>{if(closed)return;socket.send(JSON.stringify({id:1,data:'GetStatus'}));};
  socket.onmessage=event=>{if(closed)return;try{const data=JSON.parse(event.data).data;if(data&&('Status' in data||'Patch' in data)){state(true);onChange();}}catch{socket.close();}};
  socket.onerror=()=>socket.close();
  socket.onclose=()=>{state(false);if(!closed)retry=setTimeout(connect,1500);};
 }
 connect();
 return {get connected(){return connected;},stop(){closed=true;clearTimeout(retry);if(socket){socket.onclose=null;socket.close();}state(false);}};
}
