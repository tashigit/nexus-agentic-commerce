## Nexus High-Frequency Agentic Commerce (HFAC)

The landscape of autonomous trade is rapidly evolving: Stripe has launched its [Agentic Commerce Suite](https://stripe.com/blog/agentic-commerce-suite), Shopify and Google are co-developing the [Universal Commerce Protocol (UCP)](https://shopify.engineering/ucp), and Pine Labs is collaborating with OpenAI to engineer [agentic commerce in India](https://www.pinelabs.com/media-analyst/pine-labs-collaborates-with-openai-to-engineer-the-era-of-agentic-commerce-in-india).

### **The Problem: The "Middleman Tax" and the "Speed Wall"**

As AI agents begin to conduct business autonomously (buying ad space, renting compute power, or sourcing data), they hit a fundamental barrier. Traditional payment systems (like Stripe) and cloud platforms are built for humans. They are:

* **Too Slow:** Traditional settlement takes days; agentic commerce needs milliseconds.
* **Too Expensive:** Fixed fees and "platform taxes" (3-30%) eat micro-transaction margins.
* **Too Centralized:** Reliance on central intermediaries creates bottlenecks and single points of failure.

---

### **The Solution: Nexus-HFAC**

Nexus is a "headless" coordination and clearing layer that allows AI agents to negotiate and settle deals directly with each other in **under 30 milliseconds** with **zero middleman fees.**

It is built on top of **Tashi Vertex**, a leaderless coordination engine that provides the "mathematical handshake" needed for machines to trust each other without a central bank or boss.

#### **1. The Developer SDK**

A lightweight Rust library that developers plug into their AI agents. It provides:

* **Agent Identity** — Ed25519 keypairs for each agent, with `AgentId` derived from the public key. Mandates define spending limits (`max_bid`, `daily_spend_cap`) and trust thresholds.
* **Consensus Engine Abstraction** — A `ConsensusEngine` trait for ordering bids; currently backed by `SimulatedEngine`, which mimics Vertex's ~26ms BFT window with fair timestamp-based ordering.
* **Auction Resolution** — `resolve_auction` filters bids by trust score, picks the highest valid bid, and produces a `Deal` with latency metadata.
* **Clearing House** — In-memory netting ledger that accumulates deals between agent pairs and emits `Settlement` records when a configurable threshold is reached, reducing payment friction.

#### **2. The Coordination Platform**

A dashboard for business owners to oversee their autonomous workforce:

* **Mandate Control:** Set the "rules of the game" for your agents.
* **Real-Time Clearing:** View a live feed of finalized deals and "Proof of Coordination" receipts.
* **Batch Settlement:** Instead of paying for every tiny deal, the platform "batches" thousands of micro-transactions into a single final payment at the end of the day.

---

### **Key Commercial Advantages**

* **Reclaim Margins:** Eliminate the 3–30% "Platform Tax" taken by centralized marketplaces.
* **Machine-Scale Economics:** Enable new business models based on millions micro transactions.

### **The Vision**

Nexus is the **"Visa for the Agentic Web."** We aren't building a single store; we are building the infrastructure that allows every AI agent on earth to trade value as fast as they trade information.

---

### **Repository Structure**

| Component | Description |
|-----------|-------------|
| **nexus-sdk** | Provides two variants:<br>- `vertex` (Rust): Uses `tashi-vertex` to provide BFT consensus window, Ed25519 identity, auction and clearing house.<br>- `foxmq` (Python): Uses `paho-mqtt` to connect to a decentralized FoxMQ broker, leveraging its mathematically fair ordering and global message distribution underneath standard MQTT abstractions. |
| **nexus-demo** | Sandbox simulations using the SDK end-to-end to run ad auctions every 50ms, stream events over WebSocket, and handle stress commands. Exists in two variants:<br>- `vertex` (Rust): Direct Tashi Vertex integration.<br>- `foxmq` (Python): `asyncio` simulation interacting over FoxMQ. |
| **dashboard** | Svelte web app that connects to the demo's WebSocket (on port 3001). Shows live mesh topology (agent nodes and deal edges), metrics (TPS, latency, deals, settlements), transaction feed, agent panel, and simulation controls. |

---

### **Running the Demos**

You can run either the Rust-based Vertex integration or the Python-based FoxMQ abstraction. Both connect to the same web dashboard.

#### **Option A: Tashi Vertex (Rust)**

This version runs the consensus layer directly in-process via `tashi-vertex-rs`.

1. **Start the Demo (Terminal 1)**
```sh
cargo run --bin nexus-demo
```

2. **Start the Dashboard (Terminal 2)**
```sh
cd dashboard
npm install
npm run dev
```

#### **Option B: FoxMQ (Python)**

FoxMQ acts as a decentralized MQTT 5.0 broker that abstracts away Tashi Vertex. 

1. **Start the FoxMQ Broker (Terminal 1)**
Follow setup from [FoxMQ docs](https://docs.tashi.network/resources/foxmq) to start a local node:
```sh
# Download FoxMQ binary & generate keys
./foxmq address-book from-range 127.0.0.1 19793 19793
./foxmq user add --write-mode truncate
./foxmq run --secret-key-file=foxmq.d/key_0.pem --allow-anonymous-login
```


2. **Run the Demo (Terminal 2)**
```sh
cd nexus-demo/foxmq
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python main.py
```

3. **Start the Dashboard (Terminal 3)**
```sh
cd dashboard
npm install
npm run dev
```

---

### **Production Readiness: Pending Work**

The SDK is currently a **prototype** suitable for demos and local simulation. To reach production:

| Area | Status | Pending |
|------|--------|---------|
| **Vertex Integration** | `ConsensusEngine` trait exists; only `SimulatedEngine` implemented | Implement a real Vertex client backend so agents can participate in a live BFT network instead of in-process simulation |
| **Cryptographic Attestation** | Agents have Ed25519 keys; signing keys are unused | Sign bids and deals so counterparties can verify authenticity; add verification on settlement |
| **Persistence** | `ClearingHouse` is in-memory only | Durable ledger (e.g. SQLite or append-only log) for crash recovery and audit trail |
| **Wire Protocol** | None; demo uses in-process calls | Define and implement a network protocol (e.g. WebSocket or gRPC) for agents to communicate across processes |
| **Asset Types** | Ad-specific (`AdSlot`, `Bid`) | Generalize to generic asset/order types (e.g. `ComputeSlot`, `DataSlot`) for Energy, Data, Compute |
| **Settlement Execution** | `Settlement` records produced but not executed | Integrate with payment rails (on-chain or fiat) to actually move value |
| **Key Management** | Keys generated on creation; no persistence | Key derivation, secure storage, backup, and recovery for production deployments |
| **Precision** | `f64` for amounts | Use fixed-point or decimal types for money to avoid precision errors |
| **API Surface** | Low-level modules; manual wiring | High-level client API that abstracts engine, handshake, and clearing into a single flow |
