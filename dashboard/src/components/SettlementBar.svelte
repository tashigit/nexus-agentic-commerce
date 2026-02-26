<script>
  import { metrics } from '../lib/stores.js';
</script>

<div class="settlement-bar">
  <div class="bar-header">
    <span class="bar-label">SETTLEMENT CYCLE</span>
    <span class="bar-count">{$metrics.total_settled}</span>
  </div>
  <div class="bar-track">
    <div class="bar-fill" style="width: {Math.min(($metrics.total_settled % 10) / 10 * 100, 100)}%"></div>
    <div class="bar-marks">
      {#each [0,1,2,3,4,5,6,7,8,9] as _}
        <div class="mark"></div>
      {/each}
    </div>
  </div>
</div>

<style>
  .settlement-bar {
    padding: 6px 12px 12px;
  }

  .bar-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 5px;
  }

  .bar-label {
    font-family: var(--font-mono);
    font-size: 8px;
    letter-spacing: 1px;
    color: var(--text-dim);
  }

  .bar-count {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 700;
    color: var(--yellow);
  }

  .bar-track {
    height: 6px;
    background: rgba(255,255,255,0.03);
    border: 1px solid var(--border);
    position: relative;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--yellow), #ffcc44);
    transition: width 0.4s ease;
    box-shadow: 0 0 10px rgba(255, 170, 0, 0.2);
  }

  .bar-marks {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
  }

  .mark {
    flex: 1;
    border-right: 1px solid rgba(255,255,255,0.04);
  }

  .mark:last-child {
    border-right: none;
  }
</style>
