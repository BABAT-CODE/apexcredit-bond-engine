# ApexCredit Bond Engine - Quick Start (5 Minutes)

## 1. Understand What You Have

This is a **production-ready Soroban smart contract** for institutional bond issuance on Stellar.

**Core capability**: Issue tokenized corporate bonds → Redeem at maturity → Trade secondary

## 2. Review the Spec (2 minutes)

```bash
# Read the three core documents
cat requirements.md    # Business requirements
cat design.md         # Technical design
cat tasks.md          # Implementation tasks
```

**Key Numbers**:
- ✅ 9 core contract functions
- ✅ 4 data structures
- ✅ 10 error codes
- ✅ 6 query functions

## 3. Build the Contract (1 minute)

```bash
# Prerequisites: Rust, Soroban CLI
rustup target add wasm32-unknown-unknown
cargo install soroban-cli

# Build
cd contracts/apex_core
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/apex_credit.wasm
```

**Status**: ✅ Ready to deploy

## 4. Deploy to Testnet (30 seconds)

```bash
# Add your keypair
soroban keys add my-key --secret-key <your-secret>

# Deploy
CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/apex_credit.wasm \
  --source my-key \
  --network testnet)

echo "Deployed at: $CONTRACT_ID"
```

**Save the CONTRACT_ID** - you'll need it for all interactions.

## 5. Issue Your First Bond (1 minute)

```bash
# Prepare parameters
ISSUER=<your-address>
BOND_ID="BOND_2026_001"
FACE_VALUE="100000000"           # 100 USDC
MATURITY_LEDGER="5000000"        # ~1 month from now
COUPON_RATE_BPS="500"            # 5% annual
PAYMENT_TOKEN=<USDC-contract>    # USDC on Stellar
ESCROW_AMOUNT="100000000"        # Must equal face value

# Issue the bond
soroban contract invoke \
  --id $CONTRACT_ID \
  --source my-key \
  --network testnet \
  -- issue_corporate_bond \
  --issuer $ISSUER \
  --bond_id $BOND_ID \
  --face_value $FACE_VALUE \
  --maturity_ledger $MATURITY_LEDGER \
  --coupon_rate_bps $COUPON_RATE_BPS \
  --payment_token $PAYMENT_TOKEN \
  --escrow_amount $ESCROW_AMOUNT
```

**Success**: Bond tokens minted to issuer!

## Next Steps

### Check Your Bond
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- query_bond_terms \
  --bond_id "BOND_2026_001"
```

### Transfer to Another Holder
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source my-key \
  --network testnet \
  -- transfer \
  --from $ISSUER \
  --to $OTHER_ADDRESS \
  --bond_id "BOND_2026_001" \
  --amount "50000000"
```

### Redeem at Maturity
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source holder-key \
  --network testnet \
  -- redeem_principal \
  --holder <holder-address> \
  --bond_id "BOND_2026_001" \
  --amount "50000000"
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

## Available Functions

| Function | Type | Purpose |
|----------|------|---------|
| `issue_corporate_bond` | State | Issue a new bond |
| `redeem_principal` | State | Redeem bonds at maturity |
| `transfer` | State | Trade bonds secondarily |
| `query_bond_terms` | Query | Get bond parameters |
| `query_bondholder_balance` | Query | Get holder's tokens |
| `query_accrued_interest` | Query | Get accrued coupon |
| `query_escrow` | Query | Get escrow status |
| `query_total_minted` | Query | Get total issued |
| `query_total_redeemed` | Query | Get total redeemed |

## Error Codes

| Code | Meaning |
|------|---------|
| 401 | Authorization failed |
| 402 | Maturity in past |
| 403 | Invalid coupon rate |
| 404 | Insufficient escrow |
| 405 | Bond already exists |
| 406 | Bond not yet mature |
| 407 | Insufficient balance |
| 408 | Bond not found |
| 409 | Invalid recipient |
| 410 | Escrow underfunded |

## Testing

```bash
cd contracts/apex_core
cargo test

# All tests should pass before mainnet deployment
```

## Full Documentation

- **README.md** - Complete API documentation
- **ARCHITECTURE.md** - System design & security model
- **DEPLOYMENT.md** - Detailed deployment instructions
- **PRODUCTION_READY.md** - Status checklist
- **SUMMARY.md** - Executive overview

## Common Issues

### Build fails with "cargo not found"
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Deploy fails with "network not found"
```bash
# Use correct network
--network testnet    # Stellar Testnet
--network public     # Stellar Public (mainnet)
```

### Authorization error on transaction
```bash
# Ensure keypair is added to Soroban CLI
soroban keys add my-key --secret-key <secret>
soroban keys ls
```

### Transaction fails with ERR_INVALID_MATURITY
```bash
# Maturity must be in future
# Current ledger is ~60,000
# Use maturity_ledger > 60,000
```

## Next Phase

Phase 2 will add:
- Automatic yield streaming via Drips Network
- Block-by-block interest distribution
- Enhanced secondary market features

---

**Ready to go!** 🚀

For questions: team+technical@apexcredit.io
