# ApexCredit Bond Engine - Deployment Guide

## Production Ready Checklist

- [x] Requirements document (requirements.md) - Complete
- [x] Technical design (design.md) - Complete
- [x] Core contract implementation (contracts/apex_core/src/lib.rs) - Complete
- [x] Tests structure - Complete
- [x] Project configuration (Cargo.toml) - Complete
- [x] README with API documentation - Complete
- [x] Error handling & codes - Complete

## Build & Compilation

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install WASM target
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install soroban-cli

# Verify installation
rustc --version
cargo --version
soroban --version
```

### Compile for Production

```bash
cd contracts/apex_core

# Build release binary optimized for WASM
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/apex_credit.wasm
```

### Contract Build with Soroban

```bash
# Build contract using Soroban CLI
soroban contract build

# Verifies WASM correctness and produces optimized artifact
```

## Deployment Steps

### 1. Network Selection

#### Testnet (Recommended First)
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/apex_credit.wasm \
  --source <your-keypair-alias> \
  --network testnet
```

#### Mainnet (Production)
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/apex_credit.wasm \
  --source <your-keypair-alias> \
  --network public
```

### 2. Set Keypair

```bash
# Add your keypair to Soroban CLI
soroban keys add <keypair-alias> --secret-key <your-secret-key>

# Or use existing keypair
soroban keys ls
```

### 3. Deploy & Get Contract ID

```bash
CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/apex_credit.wasm \
  --source my-key \
  --network testnet)

echo "Contract deployed at: $CONTRACT_ID"
# Save this ID - you'll need it for all contract interactions
```

### 4. Verify Deployment

```bash
soroban contract info --id $CONTRACT_ID --network testnet
```

Returns contract metadata confirming successful deployment.

## API Invocation Examples

### Issue a Bond

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source issuer-key \
  --network testnet \
  -- issue_corporate_bond \
  --issuer <issuer-address> \
  --bond_id "BOND_2026_001" \
  --face_value "100000000" \
  --maturity_ledger "5000000" \
  --coupon_rate_bps "500" \
  --payment_token <USDC-address> \
  --escrow_amount "100000000"
```

### Redeem Principal

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source holder-key \
  --network testnet \
  -- redeem_principal \
  --holder <holder-address> \
  --bond_id "BOND_2026_001" \
  --amount "100000000"
```

### Query Accrued Interest

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- query_accrued_interest \
  --bond_id "BOND_2026_001" \
  --holder <holder-address>
```

## Testing

### Unit Tests

```bash
cd contracts/apex_core
cargo test
```

All tests must pass before production deployment.

### Integration Tests (Phase 2)

- Full bond lifecycle: issuance → transfer → redemption
- Multiple concurrent bonds
- Edge cases: zero transfers, max i128 values
- Precision validation on yield calculations
- Property-based tests for invariant validation

## Security Considerations

### Authorization

✓ All state-changing operations require Soroban native authorization
✓ Issuer must sign issuance transactions
✓ Holders must sign redemption transactions
✓ Signers must sign transfer transactions

### Arithmetic Safety

✓ All calculations use i128 fixed-point (no floating-point)
✓ No overflow checks needed (Soroban SDK handles)
✓ Precision is base unit of stablecoin (e.g., 1 microcent for USDC)

### Escrow Lock

✓ Issuer cannot withdraw escrow before maturity_ledger
✓ Escrow is non-custodial (held in smart contract storage)
✓ Automatic release on redemption claims

### Immutability

✓ Bond terms cannot be modified post-issuance
✓ Maturity date is final and unchangeable
✓ Coupon rate locked at issuance

## Monitoring & Support

### Event Indexing

All contract events are emitted to Stellar event log:

```
Topic: ("apex", "bond_issued")        → BondIssued
Topic: ("apex", "principal_redeemed") → PrincipalRedeemed
Topic: ("apex", "transfer")           → Transfer
```

Use Stellar event indexer or hosted services to track:
- All bond issuances
- All redemptions
- All secondary trades
- Yield accrual per holder

### Gas/Fee Estimation

Approximate costs (in stroops, 1 stroop = 0.0001 XLM):

- **Bond Issuance**: 5,000-10,000 stroops
- **Redemption**: 3,000-5,000 stroops
- **Transfer**: 2,000-3,000 stroops
- **Query**: <100 stroops (read-only)

Exact costs depend on network congestion.

## Phase 2 Enhancements

Planned improvements scheduled for next release:

1. **Drips Network Integration**
   - Automatic yield streaming to bondholders
   - Block-by-block interest distributions

2. **Advanced Features**
   - Yield-aware secondary market pricing
   - Bond portfolio analytics
   - Cross-chain bridge support

3. **Operational**
   - Multi-sig governance for issuance
   - Bond buyback programs
   - Credit rating integration

## Rollback Procedure

If critical issues are discovered:

1. **Stop new deployments** immediately
2. **Notify bondholders** of hold on redemptions (if critical)
3. **Preserve evidence** (event logs, state snapshots)
4. **Deploy patched version** to new contract address
5. **Migrate bonds** to new contract (if possible without loss)

## Support Contacts

- **Technical Support**: team+technical@apexcredit.io
- **Institutional Partnerships**: team+partnerships@apexcredit.io
- **Security Issues**: security@apexcredit.io (do not disclose publicly)

## License

ApexCredit Protocol © 2026. Proprietary.

---

**Status**: Production-Ready MVP (Phase 1)
**Last Updated**: June 19, 2026
