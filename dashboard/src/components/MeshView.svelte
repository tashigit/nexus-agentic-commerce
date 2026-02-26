<script>
  import { onMount } from 'svelte';
  import { meshNodes, meshEdges, dealPulses, nodeHeat } from '../lib/stores.js';

  let canvas;
  let ctx;
  let animFrame;
  let W = 800;
  let H = 600;
  let dpr = 1;

  // Force-directed simulation state
  let sim = []; // { id, x, y, vx, vy, role, active, targetX, targetY }
  let simMap = {};
  let hoverNode = null;
  let mouseX = -1, mouseY = -1;

  // Subscribe to store data outside of draw loop
  let currentNodes = [];
  let currentEdges = [];
  let currentPulses = [];
  let currentHeat = {};

  onMount(() => {
    dpr = window.devicePixelRatio || 1;
    ctx = canvas.getContext('2d');
    resize();
    window.addEventListener('resize', resize);
    canvas.addEventListener('mousemove', onMouse);
    canvas.addEventListener('mouseleave', () => { hoverNode = null; });

    const unsubs = [
      meshNodes.subscribe(v => { currentNodes = v; syncSimulation(v); }),
      meshEdges.subscribe(v => { currentEdges = v; }),
      dealPulses.subscribe(v => { currentPulses = v; }),
      nodeHeat.subscribe(v => { currentHeat = v; }),
    ];

    draw();

    return () => {
      window.removeEventListener('resize', resize);
      canvas.removeEventListener('mousemove', onMouse);
      if (animFrame) cancelAnimationFrame(animFrame);
      unsubs.forEach(u => u());
    };
  });

  function resize() {
    const rect = canvas.parentElement.getBoundingClientRect();
    W = rect.width;
    H = rect.height;
    dpr = window.devicePixelRatio || 1;
    canvas.width = W * dpr;
    canvas.height = H * dpr;
    canvas.style.width = W + 'px';
    canvas.style.height = H + 'px';
  }

  function onMouse(e) {
    const rect = canvas.getBoundingClientRect();
    mouseX = e.clientX - rect.left;
    mouseY = e.clientY - rect.top;
  }

  // Sync incoming server positions into our force simulation
  function syncSimulation(nodes) {
    const existing = new Set(sim.map(s => s.id));
    for (const n of nodes) {
      if (!simMap[n.id]) {
        // New node — place with slight randomization around target
        const angle = Math.random() * Math.PI * 2;
        const nudge = 20 + Math.random() * 30;
        const entry = {
          id: n.id,
          x: W / 2 + Math.cos(angle) * nudge,
          y: H / 2 + Math.sin(angle) * nudge,
          vx: 0, vy: 0,
          role: n.role,
          active: n.active,
        };
        sim.push(entry);
        simMap[n.id] = entry;
      } else {
        simMap[n.id].role = n.role;
        simMap[n.id].active = n.active;
      }
    }
  }

  // Force-directed physics tick
  function physicsTick() {
    if (sim.length === 0) return;

    const cx = W / 2, cy = H / 2;
    const k = 0.0004;         // spring constant for edges
    const repulsion = 2800;   // repulsion force
    const gravity = 0.012;    // pull toward center
    const damping = 0.88;

    // Separate publishers (inner ring) and advertisers (outer ring)
    for (const node of sim) {
      // Gravity toward center
      const dx = cx - node.x;
      const dy = cy - node.y;
      node.vx += dx * gravity;
      node.vy += dy * gravity;

      // Orbital tendency: publishers cluster closer, advertisers further
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      const targetR = node.role === 'publisher' ? Math.min(W, H) * 0.2 : Math.min(W, H) * 0.34;
      const rForce = (targetR - dist) * 0.003;
      node.vx -= (dx / dist) * rForce;
      node.vy -= (dy / dist) * rForce;
    }

    // Repulsion between all nodes
    for (let i = 0; i < sim.length; i++) {
      for (let j = i + 1; j < sim.length; j++) {
        const a = sim[i], b = sim[j];
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        const d2 = dx * dx + dy * dy || 1;
        const d = Math.sqrt(d2);
        const force = repulsion / d2;
        const fx = (dx / d) * force;
        const fy = (dy / d) * force;
        a.vx += fx; a.vy += fy;
        b.vx -= fx; b.vy -= fy;
      }
    }

    // Edge attraction
    for (const edge of currentEdges) {
      const a = simMap[edge.from];
      const b = simMap[edge.to];
      if (!a || !b) continue;
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const d = Math.sqrt(dx * dx + dy * dy) || 1;
      const strength = k * Math.min(edge.weight, 200);
      const fx = (dx / d) * strength;
      const fy = (dy / d) * strength;
      a.vx += fx; a.vy += fy;
      b.vx -= fx; b.vy -= fy;
    }

    // Integrate
    const pad = 30;
    for (const node of sim) {
      node.vx *= damping;
      node.vy *= damping;
      node.x += node.vx;
      node.y += node.vy;
      // Clamp
      node.x = Math.max(pad, Math.min(W - pad, node.x));
      node.y = Math.max(pad, Math.min(H - pad, node.y));
    }
  }

  function draw() {
    if (!ctx) { animFrame = requestAnimationFrame(draw); return; }

    const now = performance.now();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    // Background
    ctx.fillStyle = '#0a0a0f';
    ctx.fillRect(0, 0, W, H);

    // Subtle grid
    drawGrid();

    physicsTick();

    // Find hover
    hoverNode = null;
    if (mouseX >= 0) {
      for (const node of sim) {
        const dx = node.x - mouseX;
        const dy = node.y - mouseY;
        if (dx * dx + dy * dy < 400) {
          hoverNode = node;
          break;
        }
      }
    }

    if (sim.length === 0) {
      ctx.fillStyle = '#555568';
      ctx.font = '13px "JetBrains Mono", monospace';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText('Waiting for mesh data...', W / 2, H / 2);
      animFrame = requestAnimationFrame(draw);
      return;
    }

    // Draw edges
    drawEdges(now);

    // Draw pulses
    drawPulses(now);

    // Draw nodes
    drawNodes(now);

    // Draw hover tooltip
    if (hoverNode) drawTooltip(hoverNode);

    // Legend
    drawLegend();

    animFrame = requestAnimationFrame(draw);
  }

  function drawGrid() {
    ctx.strokeStyle = 'rgba(255,255,255,0.02)';
    ctx.lineWidth = 1;
    const step = 40;
    for (let x = step; x < W; x += step) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, H);
      ctx.stroke();
    }
    for (let y = step; y < H; y += step) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(W, y);
      ctx.stroke();
    }

    // Subtle radial gradient overlay for depth
    const grd = ctx.createRadialGradient(W / 2, H / 2, 0, W / 2, H / 2, Math.max(W, H) * 0.6);
    grd.addColorStop(0, 'rgba(0, 212, 170, 0.015)');
    grd.addColorStop(1, 'rgba(0, 0, 0, 0)');
    ctx.fillStyle = grd;
    ctx.fillRect(0, 0, W, H);
  }

  function drawEdges(now) {
    for (const edge of currentEdges) {
      const a = simMap[edge.from];
      const b = simMap[edge.to];
      if (!a || !b) continue;

      const intensity = Math.min(edge.weight / 80, 1);
      const alpha = 0.06 + intensity * 0.2;
      const lineW = 0.5 + intensity * 2;

      // Glow layer
      ctx.strokeStyle = `rgba(0, 212, 170, ${alpha * 0.4})`;
      ctx.lineWidth = lineW + 3;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();

      // Main line
      ctx.strokeStyle = `rgba(0, 212, 170, ${alpha})`;
      ctx.lineWidth = lineW;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();

      // Highlight edges connected to hovered node
      if (hoverNode && (edge.from === hoverNode.id || edge.to === hoverNode.id)) {
        ctx.strokeStyle = `rgba(0, 255, 204, 0.5)`;
        ctx.lineWidth = lineW + 1;
        ctx.beginPath();
        ctx.moveTo(a.x, a.y);
        ctx.lineTo(b.x, b.y);
        ctx.stroke();
      }
    }
  }

  function drawPulses(now) {
    // Clean expired from store periodically via draw — just skip expired ones
    for (const pulse of currentPulses) {
      const age = now - pulse.born;
      if (age > pulse.duration || age < 0) continue;

      const a = simMap[pulse.from];
      const b = simMap[pulse.to];
      if (!a || !b) continue;

      const t = age / pulse.duration;
      const ease = t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t; // ease-in-out
      const px = a.x + (b.x - a.x) * ease;
      const py = a.y + (b.y - a.y) * ease;

      const fadeIn = Math.min(t * 4, 1);
      const fadeOut = Math.min((1 - t) * 4, 1);
      const alpha = fadeIn * fadeOut;

      const size = 2 + Math.min(pulse.amount / 30, 4);

      // Outer glow
      const grd = ctx.createRadialGradient(px, py, 0, px, py, size * 4);
      grd.addColorStop(0, `rgba(0, 255, 204, ${alpha * 0.35})`);
      grd.addColorStop(1, 'rgba(0, 255, 204, 0)');
      ctx.fillStyle = grd;
      ctx.beginPath();
      ctx.arc(px, py, size * 4, 0, Math.PI * 2);
      ctx.fill();

      // Core
      ctx.fillStyle = `rgba(255, 255, 255, ${alpha * 0.9})`;
      ctx.beginPath();
      ctx.arc(px, py, size * 0.6, 0, Math.PI * 2);
      ctx.fill();

      // Trail — 3 fading dots behind
      for (let ti = 1; ti <= 3; ti++) {
        const tt = Math.max(0, ease - ti * 0.06);
        const tx = a.x + (b.x - a.x) * tt;
        const ty = a.y + (b.y - a.y) * tt;
        const ta = alpha * (1 - ti * 0.3);
        ctx.fillStyle = `rgba(0, 212, 170, ${ta * 0.4})`;
        ctx.beginPath();
        ctx.arc(tx, ty, size * 0.4, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }

  function drawNodes(now) {
    for (const node of sim) {
      const heat = currentHeat[node.id] || 0;
      const isHovered = hoverNode && hoverNode.id === node.id;
      const baseR = node.role === 'publisher' ? 7 : 5;
      const heatR = Math.min(heat * 0.5, 6);
      const r = baseR + heatR + (isHovered ? 3 : 0);

      let color, glowColor;
      if (!node.active) {
        color = '#ff4466';
        glowColor = 'rgba(255, 68, 102, 0.4)';
      } else if (node.role === 'publisher') {
        color = '#00d4aa';
        glowColor = 'rgba(0, 212, 170, 0.3)';
      } else {
        color = '#4488ff';
        glowColor = 'rgba(68, 136, 255, 0.3)';
      }

      // Heat ring — pulsing halo for active nodes
      if (heat > 0.5 && node.active) {
        const pulse = Math.sin(now * 0.005 + node.x) * 0.3 + 0.7;
        const haloR = r + 6 + heat * 1.5;
        const haloAlpha = Math.min(heat * 0.04, 0.3) * pulse;
        const grd = ctx.createRadialGradient(node.x, node.y, r, node.x, node.y, haloR);
        grd.addColorStop(0, color.replace(')', `, ${haloAlpha})`).replace('rgb', 'rgba').replace('##', '#'));
        // Simpler approach
        ctx.beginPath();
        ctx.arc(node.x, node.y, haloR, 0, Math.PI * 2);
        ctx.fillStyle = `${color}${Math.round(haloAlpha * 255).toString(16).padStart(2, '0')}`;
        ctx.fill();
      }

      // Outer glow
      const glowR = r + (isHovered ? 14 : 8);
      const grd = ctx.createRadialGradient(node.x, node.y, r * 0.5, node.x, node.y, glowR);
      grd.addColorStop(0, glowColor);
      grd.addColorStop(1, 'rgba(0,0,0,0)');
      ctx.fillStyle = grd;
      ctx.beginPath();
      ctx.arc(node.x, node.y, glowR, 0, Math.PI * 2);
      ctx.fill();

      // Failed node: X-cross pattern
      if (!node.active) {
        ctx.strokeStyle = 'rgba(255, 68, 102, 0.3)';
        ctx.lineWidth = 1;
        const s = r + 4;
        ctx.beginPath();
        ctx.moveTo(node.x - s, node.y - s);
        ctx.lineTo(node.x + s, node.y + s);
        ctx.moveTo(node.x + s, node.y - s);
        ctx.lineTo(node.x - s, node.y + s);
        ctx.stroke();
      }

      // Node body — publisher is hexagon, advertiser is circle
      ctx.fillStyle = color;
      if (node.role === 'publisher') {
        drawHex(node.x, node.y, r);
        ctx.fill();
        // Inner detail
        ctx.fillStyle = '#0a0a0f';
        drawHex(node.x, node.y, r * 0.4);
        ctx.fill();
      } else {
        ctx.beginPath();
        ctx.arc(node.x, node.y, r, 0, Math.PI * 2);
        ctx.fill();
        // Inner dot
        ctx.fillStyle = '#0a0a0f';
        ctx.beginPath();
        ctx.arc(node.x, node.y, r * 0.35, 0, Math.PI * 2);
        ctx.fill();
      }

      // Thin border
      ctx.strokeStyle = isHovered ? '#fff' : `${color}88`;
      ctx.lineWidth = isHovered ? 1.5 : 0.5;
      if (node.role === 'publisher') {
        drawHex(node.x, node.y, r);
        ctx.stroke();
      } else {
        ctx.beginPath();
        ctx.arc(node.x, node.y, r, 0, Math.PI * 2);
        ctx.stroke();
      }

      // Label (only show for hovered or publishers)
      if (isHovered || node.role === 'publisher') {
        ctx.fillStyle = isHovered ? 'rgba(255,255,255,0.8)' : 'rgba(136, 136, 160, 0.45)';
        ctx.font = `${isHovered ? 10 : 8}px "JetBrains Mono", monospace`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';
        ctx.fillText(node.id.slice(0, 8), node.x, node.y + r + 4);
      }
    }
  }

  function drawHex(cx, cy, r) {
    ctx.beginPath();
    for (let i = 0; i < 6; i++) {
      const angle = (Math.PI / 3) * i - Math.PI / 6;
      const x = cx + r * Math.cos(angle);
      const y = cy + r * Math.sin(angle);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.closePath();
  }

  function drawTooltip(node) {
    const px = node.x + 16;
    const py = node.y - 40;
    const heat = currentHeat[node.id] || 0;

    const lines = [
      node.id,
      `role: ${node.role}`,
      `status: ${node.active ? 'active' : 'OFFLINE'}`,
      `heat: ${heat.toFixed(1)}`,
    ];

    ctx.font = '10px "JetBrains Mono", monospace';
    const lineH = 15;
    const padX = 10, padY = 8;
    const maxW = Math.max(...lines.map(l => ctx.measureText(l).width));
    const boxW = maxW + padX * 2;
    const boxH = lines.length * lineH + padY * 2;

    // Clamp tooltip to viewport
    const tx = Math.min(px, W - boxW - 8);
    const ty = Math.max(8, Math.min(py, H - boxH - 8));

    // Background
    ctx.fillStyle = 'rgba(14, 14, 22, 0.92)';
    ctx.strokeStyle = node.active ? (node.role === 'publisher' ? '#00d4aa44' : '#4488ff44') : '#ff446644';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.roundRect(tx, ty, boxW, boxH, 3);
    ctx.fill();
    ctx.stroke();

    // Text
    ctx.textAlign = 'left';
    ctx.textBaseline = 'top';
    for (let i = 0; i < lines.length; i++) {
      ctx.fillStyle = i === 0 ? '#e0e0e8' : '#8888a0';
      if (lines[i].includes('OFFLINE')) ctx.fillStyle = '#ff4466';
      ctx.fillText(lines[i], tx + padX, ty + padY + i * lineH);
    }
  }

  function drawLegend() {
    const x = 14, y = H - 28;
    ctx.font = '9px "JetBrains Mono", monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';

    // Publisher
    ctx.fillStyle = '#00d4aa';
    drawHex(x + 5, y, 5);
    ctx.fill();
    ctx.fillStyle = '#666';
    ctx.fillText('Publisher', x + 16, y);

    // Advertiser
    const x2 = x + 90;
    ctx.fillStyle = '#4488ff';
    ctx.beginPath();
    ctx.arc(x2 + 5, y, 4, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#666';
    ctx.fillText('Advertiser', x2 + 16, y);

    // Deal pulse
    const x3 = x2 + 100;
    ctx.fillStyle = '#fff';
    ctx.beginPath();
    ctx.arc(x3 + 5, y, 2, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#666';
    ctx.fillText('Deal pulse', x3 + 14, y);
  }
</script>

<div class="mesh-container">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .mesh-container {
    flex: 1;
    overflow: hidden;
    position: relative;
    background: #0a0a0f;
  }

  canvas {
    display: block;
    cursor: crosshair;
  }
</style>
