<script>
  import { transactions } from '../lib/stores.js';
</script>

<div class="feed">
  {#each $transactions as tx, i (tx._id)}
    <div class="tx-row tx-{tx.type}" class:fresh={i === 0}>
      {#if tx.type === 'bid'}
        <span class="tx-badge bid">BID</span>
        <span class="tx-id">{tx.from.slice(0, 8)}</span>
        <span class="tx-arrow">&#8594;</span>
        <span class="tx-cat">{tx.slot?.category || '—'}</span>
        <span class="tx-amount">{tx.amount.toFixed(2)}</span>
      {:else if tx.type === 'deal'}
        <span class="tx-badge deal">DEAL</span>
        <span class="tx-id">{tx.buyer.slice(0, 6)}</span>
        <span class="tx-arrow">&#8596;</span>
        <span class="tx-id">{tx.seller.slice(0, 6)}</span>
        <span class="tx-amount deal-amount">{tx.amount.toFixed(2)}</span>
        <span class="tx-meta">{tx.latency_ms.toFixed(0)}ms</span>
      {:else if tx.type === 'settlement'}
        <span class="tx-badge settle">NET</span>
        <span class="tx-id">{tx.between[0]?.slice(0, 6)}</span>
        <span class="tx-arrow">&#8596;</span>
        <span class="tx-id">{tx.between[1]?.slice(0, 6)}</span>
        <span class="tx-amount settle-amount">{tx.net_amount.toFixed(2)}</span>
        <span class="tx-meta">{tx.tx_count}tx</span>
      {/if}
    </div>
  {/each}

  {#if $transactions.length === 0}
    <div class="empty">Waiting for transactions...</div>
  {/if}
</div>

<style>
  .feed {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

  .tx-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    font-family: var(--font-mono);
    font-size: 11px;
    border-bottom: 1px solid rgba(255,255,255,0.03);
    animation: fadeSlide 0.3s ease-out;
  }

  @keyframes fadeSlide {
    from {
      opacity: 0;
      transform: translateY(-6px);
      background: rgba(0, 212, 170, 0.06);
    }
    to {
      opacity: 1;
      transform: translateY(0);
      background: transparent;
    }
  }

  .tx-row:hover {
    background: rgba(255,255,255,0.025);
  }

  .tx-badge {
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.8px;
    padding: 2px 5px;
    flex-shrink: 0;
    min-width: 34px;
    text-align: center;
  }

  .tx-badge.bid {
    color: var(--blue);
    background: var(--blue-dim);
  }

  .tx-badge.deal {
    color: #0a0a0f;
    background: var(--accent);
    font-weight: 800;
  }

  .tx-badge.settle {
    color: #0a0a0f;
    background: var(--yellow);
    font-weight: 800;
  }

  .tx-id {
    color: var(--text-secondary);
    font-size: 10px;
  }

  .tx-arrow {
    color: var(--text-dim);
    font-size: 10px;
  }

  .tx-cat {
    color: var(--text-dim);
    font-size: 10px;
    opacity: 0.6;
  }

  .tx-amount {
    color: var(--text-secondary);
    margin-left: auto;
    font-size: 11px;
  }

  .tx-amount::before {
    content: '$';
    color: var(--text-dim);
    font-size: 9px;
  }

  .tx-amount.deal-amount {
    color: var(--accent);
    font-weight: 600;
  }

  .tx-amount.settle-amount {
    color: var(--yellow);
    font-weight: 600;
  }

  .tx-meta {
    color: var(--text-dim);
    font-size: 9px;
    min-width: 30px;
    text-align: right;
  }

  .empty {
    color: var(--text-dim);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 20px;
    text-align: center;
  }
</style>
