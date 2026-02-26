<script>
  import { onMount } from 'svelte';
  import { connect } from './lib/ws.js';
  import { connected } from './lib/stores.js';
  import MeshView from './components/MeshView.svelte';
  import MetricsPanel from './components/MetricsPanel.svelte';
  import TransactionFeed from './components/TransactionFeed.svelte';
  import AgentPanel from './components/AgentPanel.svelte';
  import Controls from './components/Controls.svelte';
  import SettlementBar from './components/SettlementBar.svelte';

  onMount(() => {
    connect();
  });
</script>

<div class="layout">
  <header>
    <div class="logo">
      <span class="logo-mark">&#9670;</span>
      <span class="logo-text">NEXUS</span>
      <span class="logo-sep"></span>
      <span class="logo-sub">HFC</span>
    </div>
    <div class="header-center">
      <span class="tagline">HIGH-FREQUENCY COMMERCE</span>
      <span class="subtitle">TCP/IP for Value &middot; Tashi Vertex Consensus</span>
    </div>
    <div class="status" class:online={$connected}>
      <span class="status-dot"></span>
      <span class="status-text">{$connected ? 'LIVE' : 'OFFLINE'}</span>
    </div>
  </header>

  <div class="grid">
    <div class="panel mesh-panel">
      <div class="panel-header">
        <span class="ph-dot mesh-dot"></span>
        MESH TOPOLOGY
      </div>
      <MeshView />
    </div>

    <div class="panel metrics-panel">
      <div class="panel-header">
        <span class="ph-dot metrics-dot"></span>
        LIVE METRICS
      </div>
      <MetricsPanel />
    </div>

    <div class="panel feed-panel">
      <div class="panel-header">
        <span class="ph-dot feed-dot"></span>
        TRANSACTION FEED
      </div>
      <TransactionFeed />
    </div>

    <div class="panel agents-panel">
      <div class="panel-header">
        <span class="ph-dot agents-dot"></span>
        AGENTS
      </div>
      <AgentPanel />
    </div>

    <div class="panel controls-panel">
      <div class="panel-header">
        <span class="ph-dot controls-dot"></span>
        SIMULATION
      </div>
      <Controls />
      <SettlementBar />
    </div>
  </div>
</div>

<style>
  .layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-primary);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .logo-mark {
    color: var(--accent);
    font-size: 14px;
  }

  .logo-text {
    font-family: var(--font-mono);
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: 4px;
  }

  .logo-sep {
    width: 1px;
    height: 14px;
    background: var(--border);
    margin: 0 2px;
  }

  .logo-sub {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 2px;
  }

  .header-center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
  }

  .tagline {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 5px;
    color: var(--text-dim);
  }

  .subtitle {
    font-size: 9px;
    color: var(--text-dim);
    opacity: 0.5;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 1px;
    color: var(--red);
  }

  .status.online {
    color: var(--accent);
  }

  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--red);
    animation: pulse-red 2s ease-in-out infinite;
  }

  .status.online .status-dot {
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent);
    animation: pulse-green 1.5s ease-in-out infinite;
  }

  @keyframes pulse-green {
    0%, 100% { box-shadow: 0 0 4px rgba(0, 212, 170, 0.4); }
    50% { box-shadow: 0 0 10px rgba(0, 212, 170, 0.8); }
  }

  @keyframes pulse-red {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 300px 300px;
    grid-template-rows: auto 1fr auto;
    gap: 1px;
    flex: 1;
    overflow: hidden;
    background: var(--border);
  }

  .panel {
    background: var(--bg-primary);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--text-dim);
    padding: 7px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(255,255,255,0.008);
  }

  .ph-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .mesh-dot { background: var(--accent); }
  .metrics-dot { background: var(--accent); }
  .feed-dot { background: var(--blue); }
  .agents-dot { background: var(--text-dim); }
  .controls-dot { background: var(--yellow); }

  .mesh-panel {
    grid-row: 1 / 4;
    grid-column: 1;
  }

  .metrics-panel {
    grid-column: 2 / 4;
    grid-row: 1;
  }

  .feed-panel {
    grid-column: 2;
    grid-row: 2 / 4;
  }

  .agents-panel {
    grid-column: 3;
    grid-row: 2;
  }

  .controls-panel {
    grid-column: 3;
    grid-row: 3;
  }
</style>
