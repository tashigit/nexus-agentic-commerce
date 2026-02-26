import { connected, handleEvent } from './stores.js';

let ws = null;
let reconnectTimer = null;

export function connect(url = 'ws://localhost:3001') {
  if (ws) {
    ws.close();
  }

  ws = new WebSocket(url);

  ws.onopen = () => {
    console.log('[ws] connected');
    connected.set(true);
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
  };

  ws.onmessage = (e) => {
    try {
      const event = JSON.parse(e.data);
      handleEvent(event);
    } catch (err) {
      console.warn('[ws] bad message:', err);
    }
  };

  ws.onclose = () => {
    console.log('[ws] disconnected');
    connected.set(false);
    reconnectTimer = setTimeout(() => connect(url), 2000);
  };

  ws.onerror = (err) => {
    console.error('[ws] error:', err);
    ws.close();
  };
}

export function sendCommand(cmd) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ cmd }));
  }
}
