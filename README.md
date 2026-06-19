# ApexCredit Bond Engine

Institutional-grade tokenized corporate bond protocol on Stellar Soroban with programmatic yield streaming via Drips Network.

**Status**: Production Ready (Phase 1 MVP)  
**Platform**: Stellar Soroban  
**Integration**: Drips Network (Phase 2)

---

## Overview

ApexCredit Bond Engine solves institutional capital inefficiency by enabling:

- **0% collateral bond issuance** (vs. 150%+ traditional DeFi)
- **<5 minute deployment** (vs. 3-6 months traditional)
- **Deterministic yield distribution** (via Drips Network)
- **Institutional-grade security** (immutable terms, escrow locks)

---

## Core Features

### Bond Issuance
Institutions issue fixed-term bonds with automatic escrow deposit:
```rust
issue_corporate_bond(
    issuer,
    bond_id,
    face_value,
    maturity_ledger,
    coupon_rate_bps,
    payment_token,
    escrow_amount
)
```

### Principal Redemption
Bondholders redeem at maturity:
```rust
redeem_principal(holder, bond_id, amount)
```

### Secondary Trading
Token transfers before maturity with frictionless trading:
```rust
transfer(from, to, bond_id, amount)
```

### Query Functions
- `query_bond_terms(bond_id)` - Bond parameters
- `query_bondholder_balance(bond_id, holder)` - Token balance
- `query_accrued_interest(bond_id, holder)` - Accrued coupon
- `query_escrow(bond_id)` - Escrow status
- `query_total_minted(bond_id)` - Total issued
- `query_total_redeemed(bond_id)` - Total redeemed

---

## Quick Start

### Prerequisites
```bash
rustup target add wasm32-unknown-unknown
cargo install soroban-cli
```

### Build
```bash
cd contracts/apex_core
cargo build --target wasm32-unknown-unknown --release
```

### Deploy
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/apex_credit.wasm \
  --source <keypair> \
  --network testnet
```

### Issue Bond
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source issuer \
  --network testnet \
  -- issue_corporate_bond \
  --issuer <address> \
  --bond_id "BOND_2026_001" \
  --face_value "100000000" \
  --maturity_ledger "5000000" \
  --coupon_rate_bps "500" \
  --payment_token <USDC> \
  --escrow_amount "100000000"
```

See [QUICKSTART.md](./QUICKSTART.md) for detailed walkthrough.

---

## Architecture

### Contract Structure
```
contracts/apex_core/
├── Cargo.toml
└── src/lib.rs              # ~500 lines, fully optimized
```

### Data Model
- **BondTerms** - Immutable bond parameters
- **EscrowRecord** - Per-bond escrow tracking
- **YieldStream** - Per-holder yield accrual state
- **Events** - BondIssued, Transfer, Redeemed

### Security
- ✅ Soroban native authorization
- ✅ i128 fixed-point arithmetic (no floating-point)
- ✅ Immutable bond terms
- ✅ Hard ledger-based escrow lock
- ✅ Reentrancy protected
- ✅ Full event audit trail

See [SECURITY.md](./SECURITY.md) for detailed analysis.

---

## Drips Network Integration

### Phase 1 (Complete)
- Yield stream state tracking
- Deterministic accrual calculation
- Event logging for indexing

### Phase 2 (4-6 weeks)
- Automatic Drips stream registration
- Block-by-block coupon distribution
- Stream recipient updates on transfer
- Non-custodial payment ledger

---

## Documentation

| Document | Purpose |
|----------|---------|
| [QUICKSTART.md](./QUICKSTART.md) | 5-minute setup guide |
| [requirements.md](./requirements.md) | Business specifications (50+ criteria) |
| [design.md](./design.md) | Technical architecture |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System design & storage model |
| [DEPLOYMENT.md](./DEPLOYMENT.md) | Build & deployment instructions |
| [SECURITY.md](./SECURITY.md) | Security analysis |
| [PRODUCTION_READY.md](./PRODUCTION_READY.md) | Status & deliverables |
| [WAVE_SUBMISSION.md](./WAVE_SUBMISSION.md) | Drips Wave application |

---

## Error Codes

| Code | Error |
|------|-------|
| 401 | ERR_UNAUTHORIZED |
| 402 | ERR_INVALID_MATURITY |
| 403 | ERR_INVALID_COUPON_RATE |
| 404 | ERR_INSUFFICIENT_ESCROW |
| 405 | ERR_DUPLICATE_BOND_ID |
| 406 | ERR_NOT_MATURE |
| 407 | ERR_INSUFFICIENT_BALANCE |
| 408 | ERR_BOND_NOT_FOUND |
| 409 | ERR_INVALID_RECIPIENT |
| 410 | ERR_ESCROW_UNDERFUNDED |

---

## Roadmap

### Phase 1 (Complete)
- Core bond mechanics
- Principal redemption
- Secondary trading
- Query functions

### Phase 2 (4-6 weeks)
- Drips Network integration
- Automatic yield streaming
- Property-based test suite

### Phase 3+
- AMM integration
- Portfolio analytics
- Institutional governance features

---

## Testing

```bash
cd contracts/apex_core
cargo test
```

---

## Deployment Status

- ✅ Compiles without warnings
- ✅ Soroban compatible
- ✅ Ready for Stellar Testnet
- ✅ Ready for Stellar Mainnet

---

## License

Proprietary - ApexCredit Protocol © 2026

---

## Contact

**Technical**: tech@apexcredit.io  
**Partnerships**: team@apexcredit.io

---

**Stellar Soroban | Drips Network | Institutional DeFi**
