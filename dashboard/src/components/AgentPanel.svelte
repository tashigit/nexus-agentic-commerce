<script>
  import { agents } from '../lib/stores.js';

  let showCount = 16;

  function trustBar(score) {
    return Math.max(0, Math.min(100, score * 100));
  }
</script>

<div class="agents-list">
  {#each $agents.slice(0, showCount) as agent (agent.id)}
    <div class="agent-card" class:inactive={!agent.active}>
      <div class="agent-row">
        <span class="agent-dot" class:pub={agent.role === 'publisher'} class:adv={agent.role === 'advertiser'} class:off={!agent.active}></span>
        <span class="agent-id">{agent.id.slice(0, 10)}</span>
        <span class="agent-role" class:publisher={agent.role === 'publisher'}>
          {agent.role === 'publisher' ? 'PUB' : 'ADV'}
        </span>
        {#if !agent.active}
          <span class="off-tag">OFF</span>
        {/if}
      </div>
      <div class="agent-bar-row">
        <span class="bar-label">TRUST</span>
        <div class="bar-track">
          <div class="bar-fill" class:low={agent.trust_score < 0.5} style="width: {trustBar(agent.trust_score)}%"></div>
        </div>
        <span class="bar-val">{agent.trust_score.toFixed(2)}</span>
      </div>
    </div>
  {/each}

  {#if $agents.length > showCount}
    <button class="show-more" onclick={() => showCount += 16}>
      +{$agents.length - showCount} more agents
    </button>
  {/if}

  {#if $agents.length === 0}
    <div class="empty">No agents yet...</div>
  {/if}
</div>

<style>
  .agents-list {
    flex: 1;
    overflow-y: auto;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .agent-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    padding: 6px 8px;
    transition: opacity 0.3s, border-color 0.3s;
  }

  .agent-card.inactive {
    opacity: 0.4;
    border-color: var(--red-dim);
  }

  .agent-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .agent-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .agent-dot.pub { background: var(--accent); box-shadow: 0 0 4px var(--accent); }
  .agent-dot.adv { background: var(--blue); box-shadow: 0 0 4px var(--blue); }
  .agent-dot.off { background: var(--red); box-shadow: 0 0 4px var(--red); }

  .agent-id {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .agent-role {
    font-family: var(--font-mono);
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.8px;
    color: var(--blue);
    margin-left: auto;
  }

  .agent-role.publisher {
    color: var(--accent);
  }

  .off-tag {
    font-family: var(--font-mono);
    font-size: 7px;
    font-weight: 800;
    letter-spacing: 0.5px;
    color: var(--red);
    background: var(--red-dim);
    padding: 0 4px;
  }

  .agent-bar-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
  }

  .bar-label {
    font-family: var(--font-mono);
    font-size: 7px;
    letter-spacing: 0.8px;
    color: var(--text-dim);
    min-width: 28px;
  }

  .bar-track {
    flex: 1;
    height: 3px;
    background: rgba(255,255,255,0.05);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.5s ease;
  }

  .bar-fill.low {
    background: var(--red);
  }

  .bar-val {
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-dim);
    min-width: 28px;
    text-align: right;
  }

  .show-more {
    background: none;
    border: 1px dashed var(--border);
    color: var(--text-dim);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 6px;
    cursor: pointer;
    text-align: center;
    transition: all 0.15s;
  }

  .show-more:hover {
    border-color: var(--text-secondary);
    color: var(--text-secondary);
  }

  .empty {
    color: var(--text-dim);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 20px;
    text-align: center;
  }
</style>
