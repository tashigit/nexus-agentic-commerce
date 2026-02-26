<script>
  import { sendCommand } from '../lib/ws.js';
  import { connected } from '../lib/stores.js';

  let surgeActive = false;
  let attackActive = false;
  let healActive = false;

  function fire(cmd) {
    sendCommand(cmd);
    if (cmd === 'surge') { surgeActive = true; setTimeout(() => surgeActive = false, 1500); }
    if (cmd === 'attack') { attackActive = true; setTimeout(() => attackActive = false, 1500); }
    if (cmd === 'heal') { healActive = true; setTimeout(() => healActive = false, 1500); }
  }
</script>

<div class="controls">
  <button
    class="btn surge"
    class:active={surgeActive}
    onclick={() => fire('surge')}
    disabled={!$connected}
  >
    <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
    </svg>
    <span class="btn-text">SURGE</span>
  </button>

  <button
    class="btn attack"
    class:active={attackActive}
    onclick={() => fire('attack')}
    disabled={!$connected}
  >
    <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10" />
      <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
    </svg>
    <span class="btn-text">ATTACK</span>
  </button>

  <button
    class="btn heal"
    class:active={healActive}
    onclick={() => fire('heal')}
    disabled={!$connected}
  >
    <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
    <span class="btn-text">HEAL</span>
  </button>
</div>

<style>
  .controls {
    display: flex;
    gap: 6px;
    padding: 10px 12px;
  }

  .btn {
    flex: 1;
    padding: 10px 6px;
    font-family: var(--font-mono);
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    position: relative;
    overflow: hidden;
  }

  .btn::after {
    content: '';
    position: absolute;
    inset: 0;
    opacity: 0;
    transition: opacity 0.3s;
  }

  .btn:disabled {
    opacity: 0.25;
    cursor: not-allowed;
  }

  .btn-icon {
    width: 18px;
    height: 18px;
  }

  .btn-text {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 1.5px;
  }

  .btn.surge {
    color: var(--accent);
    border-color: rgba(0, 212, 170, 0.15);
  }
  .btn.surge:hover:not(:disabled) {
    border-color: var(--accent);
    background: rgba(0, 212, 170, 0.06);
  }
  .btn.surge.active {
    background: rgba(0, 212, 170, 0.15);
    box-shadow: 0 0 20px rgba(0, 212, 170, 0.15);
  }

  .btn.attack {
    color: var(--red);
    border-color: rgba(255, 68, 102, 0.15);
  }
  .btn.attack:hover:not(:disabled) {
    border-color: var(--red);
    background: rgba(255, 68, 102, 0.06);
  }
  .btn.attack.active {
    background: rgba(255, 68, 102, 0.15);
    box-shadow: 0 0 20px rgba(255, 68, 102, 0.15);
  }

  .btn.heal {
    color: var(--blue);
    border-color: rgba(68, 136, 255, 0.15);
  }
  .btn.heal:hover:not(:disabled) {
    border-color: var(--blue);
    background: rgba(68, 136, 255, 0.06);
  }
  .btn.heal.active {
    background: rgba(68, 136, 255, 0.15);
    box-shadow: 0 0 20px rgba(68, 136, 255, 0.15);
  }
</style>
