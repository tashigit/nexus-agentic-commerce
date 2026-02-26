import { writable } from 'svelte/store';

export const connected = writable(false);
export const agents = writable([]);
export const metrics = writable({
  tps: 0,
  avg_latency_ms: 0,
  total_deals: 0,
  total_settled: 0,
  active_agents: 0,
});
export const transactions = writable([]);
export const meshNodes = writable([]);
export const meshEdges = writable([]);

// Animated deal pulses for the mesh view — particles that travel buyer→seller
export const dealPulses = writable([]);

// Track per-node activity heat (recent deal count)
export const nodeHeat = writable({});

const MAX_TRANSACTIONS = 200;
const MAX_PULSES = 60;

let txIdCounter = 0;

export function handleEvent(event) {
  switch (event.type) {
    case 'agent_created':
      agents.update(list => [...list, event.agent]);
      break;

    case 'bid':
      transactions.update(list => {
        const next = [{ _id: ++txIdCounter, type: 'bid', from: event.from, amount: event.amount, slot: event.slot, timestamp: event.timestamp }, ...list];
        return next.slice(0, MAX_TRANSACTIONS);
      });
      break;

    case 'deal':
      transactions.update(list => {
        const next = [{ _id: ++txIdCounter, type: 'deal', buyer: event.buyer, seller: event.seller, amount: event.amount, latency_ms: event.latency_ms, slot: event.slot }, ...list];
        return next.slice(0, MAX_TRANSACTIONS);
      });
      // Spawn a pulse particle for the mesh
      dealPulses.update(pulses => {
        const next = [...pulses, {
          from: event.buyer,
          to: event.seller,
          amount: event.amount,
          born: performance.now(),
          duration: 800 + Math.random() * 400,
        }];
        return next.slice(-MAX_PULSES);
      });
      // Bump heat on both nodes
      nodeHeat.update(h => {
        const copy = { ...h };
        copy[event.buyer] = (copy[event.buyer] || 0) + 1;
        copy[event.seller] = (copy[event.seller] || 0) + 1;
        return copy;
      });
      break;

    case 'settlement':
      transactions.update(list => {
        const next = [{ _id: ++txIdCounter, type: 'settlement', between: event.between, net_amount: event.net_amount, tx_count: event.tx_count }, ...list];
        return next.slice(0, MAX_TRANSACTIONS);
      });
      break;

    case 'metrics':
      metrics.set(event);
      break;

    case 'mesh_update':
      meshNodes.set(event.nodes);
      meshEdges.set(event.edges);
      break;

    case 'agent_failed':
      agents.update(list =>
        list.map(a => a.id === event.agent ? { ...a, active: false } : a)
      );
      break;

    case 'agent_healed':
      agents.update(list =>
        list.map(a => a.id === event.agent ? { ...a, active: true } : a)
      );
      break;
  }
}

// Decay node heat periodically
setInterval(() => {
  nodeHeat.update(h => {
    const copy = {};
    for (const [k, v] of Object.entries(h)) {
      const next = v * 0.85;
      if (next > 0.1) copy[k] = next;
    }
    return copy;
  });
}, 200);
