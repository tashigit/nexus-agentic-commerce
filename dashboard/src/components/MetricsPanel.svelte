<script>
  import { onMount } from 'svelte';
  import { metrics } from '../lib/stores.js';

  // Keep history for sparklines
  let tpsHistory = [];
  let latencyHistory = [];
  const MAX_HISTORY = 40;

  let prev = { tps: 0, total_deals: 0, total_settled: 0 };

  // Track deltas for flash animation
  let dealsDelta = 0;
  let settledDelta = 0;
  let dealsFlash = false;
  let settledFlash = false;

  const unsub = metrics.subscribe(m => {
    tpsHistory = [...tpsHistory, m.tps].slice(-MAX_HISTORY);
    latencyHistory = [...latencyHistory, m.avg_latency_ms].slice(-MAX_HISTORY);
    if (m.total_deals > prev.total_deals) {
      dealsDelta = m.total_deals - prev.total_deals;
      dealsFlash = true;
      setTimeout(() => dealsFlash = false, 300);
    }
    if (m.total_settled > prev.total_settled) {
      settledDelta = m.total_settled - prev.total_settled;
      settledFlash = true;
      setTimeout(() => settledFlash = false, 300);
    }
    prev = { tps: m.tps, total_deals: m.total_deals, total_settled: m.total_settled };
  });

  function sparkline(data, w, h) {
    if (data.length < 2) return '';
    const max = Math.max(...data, 1);
    const step = w / (data.length - 1);
    return data.map((v, i) => {
      const x = i * step;
      const y = h - (v / max) * h;
      return `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`;
    }).join(' ');
  }
</script>

<div class="metrics">
  <div class="metric highlight">
    <div class="metric-top">
      <span class="metric-value big">{$metrics.tps.toFixed(1)}</span>
      <span class="metric-label">TPS</span>
    </div>
    <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
      <path d={sparkline(tpsHistory, 120, 28)} fill="none" stroke="rgba(0,212,170,0.4)" stroke-width="1.5" />
    </svg>
  </div>

  <div class="metric">
    <div class="metric-top">
      <span class="metric-value">{$metrics.avg_latency_ms.toFixed(1)}<span class="unit">ms</span></span>
      <span class="metric-label">LATENCY</span>
    </div>
    <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
      <path d={sparkline(latencyHistory, 120, 28)} fill="none" stroke="rgba(68,136,255,0.4)" stroke-width="1.5" />
    </svg>
  </div>

  <div class="metric" class:flash-green={dealsFlash}>
    <span class="metric-value">{$metrics.total_deals.toLocaleString()}</span>
    <span class="metric-label">DEALS</span>
  </div>

  <div class="metric" class:flash-yellow={settledFlash}>
    <span class="metric-value settled">{$metrics.total_settled.toLocaleString()}</span>
    <span class="metric-label">SETTLED</span>
  </div>

  <div class="metric">
    <span class="metric-value agents-val">{$metrics.active_agents}<span class="unit">/ 50</span></span>
    <span class="metric-label">ACTIVE</span>
  </div>
</div>

<style>
  .metrics {
    display: flex;
    padding: 10px 10px;
    gap: 6px;
  }

  .metric {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 10px 8px 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    position: relative;
    overflow: hidden;
    transition: border-color 0.3s;
  }

  .metric.highlight {
    border-color: rgba(0, 212, 170, 0.2);
  }

  .metric.flash-green {
    border-color: var(--accent) !important;
    box-shadow: inset 0 0 20px rgba(0, 212, 170, 0.08);
  }

  .metric.flash-yellow {
    border-color: var(--yellow) !important;
    box-shadow: inset 0 0 20px rgba(255, 170, 0, 0.08);
  }

  .metric-top {
    display: flex;
    flex-direction: column;
    align-items: center;
    z-index: 1;
  }

  .metric-value {
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 700;
    color: var(--accent);
    line-height: 1;
  }

  .metric-value.big {
    font-size: 26px;
  }

  .metric-value.settled {
    color: var(--yellow);
  }

  .metric-value.agents-val {
    color: var(--text-primary);
  }

  .unit {
    font-size: 11px;
    font-weight: 400;
    color: var(--text-dim);
    margin-left: 2px;
  }

  .metric-label {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 500;
    letter-spacing: 1.5px;
    color: var(--text-dim);
    margin-top: 5px;
  }

  .spark {
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 28px;
    opacity: 0.8;
  }
</style>
