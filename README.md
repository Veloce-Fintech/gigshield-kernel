# GigShield Kernel

Automated escrow custody engine, payment path router, and compliance handler for the GigShield freelance platform on Stellar.

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │           GigShield Kernel           │
                    ├─────────────────────────────────────┤
                    │  Express Backend (API Gateway)       │
                    │  ┌──────────┐  ┌──────────────────┐ │
                    │  │ SEP-24   │  │ Path Payment      │ │
                    │  │ Adapter  │  │ Router (DEX)      │ │
                    │  └────┬─────┘  └────────┬─────────┘ │
                    └───────┼──────────────────┼───────────┘
                            │                  │
              ┌─────────────┼──────────────────┼──────────┐
              │             │                  │          │
     ┌────────▼────┐  ┌────▼─────┐    ┌───────▼──────┐   │
     │  Escrow     │  │Compliance│    │  Stellar      │   │
     │  Vault      │  │ Guard    │    │  Anchors      │   │
     │  (milestone │  │(clawback,│    │ (Yellow Card, │   │
     │   escrow)   │  │ key leak │    │  Anclap)      │   │
     └─────────────┘  └──────────┘    └──────────────┘   │
                    Soroban Contracts      SEP-24 On/Off-Ramp
```

## Repo Structure

```
gigshield-kernel/
├── contracts/
│   ├── escrow_vault/          # Soroban: milestone-based escrow with arbitration
│   │   └── src/
│   │       ├── lib.rs         # initialize, add_milestone, complete, approve, dispute, resolve
│   │       └── test.rs
│   └── compliance_guard/      # Soroban: clawback, auth flags, key recovery
│       └── src/
│           ├── lib.rs         # register, freeze, clawback, key recovery flow
│           └── test.rs
├── server/
│   └── src/
│       ├── index.ts           # Express entry — mounts routes
│       ├── adapters/
│       │   └── sep24.ts       # SEP-24 anchor deposit/withdraw adapter
│       ├── routes/
│       │   └── path-payments.ts  # DEX strict send path payment router
│       └── types/
│           └── index.ts
├── Cargo.toml                 # Rust workspace
├── Makefile
└── .env.example
```

## Contracts

### escrow_vault

Holds milestone-based freelance project funds in stablecoins (USDC/PYUSD).

| Function | Description |
|---|---|
| `initialize(client, freelancer, arbitrator, token, total)` | Deploy escrow with parties and token |
| `add_milestone(description, amount, deadline)` | Add a milestone with funds allocation |
| `complete_milestone(index)` | Freelancer marks milestone done |
| `approve_milestone(index)` | Client releases payment for completed milestone |
| `raise_dispute(reason, proposed_split)` | Client or freelancer triggers arbitration |
| `resolve_dispute(client_amt, freelancer_amt)` | Arbitrator splits funds |
| `release_deadline_locked(index)` | Arbitrator releases funds after deadline passes |

### compliance_guard

Interfaces with Stellar asset controls (Clawbacks, Auth Required) to protect business accounts from compromised key leaks.

| Function | Description |
|---|---|
| `register_account(token, signers, min_approvals)` | Register a guarded business account |
| `freeze_account(business)` | Freeze a business (signer multi-sig) |
| `clawback(business, from, amount)` | Recover assets from frozen account |
| `request_key_recovery(business, new_key)` | Initiate signer key rotation |
| `approve_recovery(business)` | Multi-sig approval for key recovery |

## Backend

### SEP-24 Anchor Adapter

On/off-ramp engine that interfaces with local Stellar Anchors:
- **Yellow Card** — Africa (NG, GH, KE, ZA, UG)
- **Anclap** — LATAM (AR, BR, CL, CO, MX)

| Endpoint | Description |
|---|---|
| `GET /api/v1/anchors` | List anchors (filterable by region) |
| `POST /api/v1/anchors/deposit` | Fiat → stablecoin on-ramp |
| `POST /api/v1/anchors/withdraw` | Stablecoin → fiat off-ramp |
| `GET /api/v1/anchors/transaction/:id` | Check transaction status |

### Path Payment Router

Uses the Stellar DEX to split a single payment batch into multiple target currencies simultaneously.

| Endpoint | Description |
|---|---|
| `POST /api/v1/path-payments/find-paths` | Discover DEX paths for multi-currency split |
| `POST /api/v1/path-payments/execute` | Submit batched path payments on-chain |

## Getting Started

```bash
# Install backend deps
cd server && npm install

# Copy env and fill in config
cp .env.example .env

# Build contracts
cd .. && make contracts

# Run backend
cd server && npm run dev
```

## Environment

| Variable | Description |
|---|---|
| `STELLAR_NETWORK` | `testnet` or `mainnet` |
| `HORIZON_URL` | Horizon RPC endpoint |
| `RPC_URL` | Soroban RPC endpoint |
| `ANCHOR_YELLOWCARD_API` | Yellow Card API base URL |
| `ANCHOR_ANCLAP_API` | Anclap API base URL |
| `ESCROW_VAULT_ID` | Deployed escrow_vault contract ID |
| `COMPLIANCE_GUARD_ID` | Deployed compliance_guard contract ID |
| `SIGNER_SECRET_KEY` | Stellar secret key for signing txs |
