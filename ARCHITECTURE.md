# ApexCredit Bond Engine - Architecture Document

## System Overview

```
┌─────────────────────────────────────────────────────────┐
│          Stellar Soroban Smart Contract Layer             │
│                                                           │
│  ApexCreditEngine Contract (lib.rs)                      │
│                                                           │
│  Functions:                                              │
│  ├─ issue_corporate_bond()                              │
│  ├─ redeem_principal()                                  │
│  ├─ transfer()                                          │
│  ├─ query_bond_terms()                                  │
│  ├─ query_bondholder_balance()                          │
│  ├─ query_accrued_interest()                            │
│  ├─ query_escrow()                                      │
│  ├─ query_total_minted()                                │
│  └─ query_total_redeemed()                              │
│                                                           │
└─────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
    Stablecoin           Bond Token           Events
    Contract            Ledger                Log
    (USDC/EURC)        (on chain)        (off-chain indexing)
```

## Storage Model

### Persistent Storage Architecture

The contract uses Soroban persistent storage with the following key patterns:

#### 1. Bond Terms Storage

```
Key: bond_id (Symbol)
Value: BondTerms struct
├─ issuer: Address
├─ bond_id: Symbol
├─ face_value: i128
├─ maturity_ledger: u32
├─ coupon_rate_bps: u32
├─ payment_token: Address
└─ issued_ledger: u32
```

**Immutable**: Once set, never modified.
**Lifetime**: From issuance → end of time (permanent record)

#### 2. Escrow Records

```
Key: "escrow_{bond_id}"
Value: EscrowRecord struct
├─ bond_id: Symbol
├─ issuer: Address
├─ escrow_balance: i128 (current)
├─ required_balance: i128 (face_value)
├─ created_ledger: u32
└─ maturity_ledger: u32
```

**Mutable**: escrow_balance decreases with each redemption
**Updated on**: Every redemption call
**Locked until**: maturity_ledger reached

#### 3. Bondholder Balances

```
Key: "balance_{bond_id}_{holder_address}"
Value: i128 (token balance)
```

**Tracking**: Per bondholder, per bond
**Updated on**: issuance, transfer, redemption
**Total invariant**: sum(all balances) = total_minted - total_redeemed

#### 4. Yield Stream State

```
Key: "yield_stream_{bond_id}_{holder_address}"
Value: YieldStream struct
├─ bond_id: Symbol
├─ holder: Address
├─ accrual_start_ledger: u32
└─ last_collection_ledger: u32
```

**Purpose**: Track per-holder yield accrual for off-chain calculations
**Updated on**: issuance, transfer
**Removed on**: full redemption

#### 5. Aggregated State

```
Key: "total_minted_{bond_id}"
Value: i128 (cumulative issued tokens)

Key: "total_redeemed_{bond_id}"
Value: i128 (cumulative redeemed tokens)
```

**Used for**: Validation, analytics, invariant checking

### Storage Optimization

- **Key naming**: Uses format strings to avoid collision (`"key_name_{id}_{addr}"`)
- **Data structure packing**: All fields in structs are necessary (no waste)
- **Lazy evaluation**: Accrued interest calculated on-query (not stored)
- **Persistent only**: No transient storage (all data is permanent record)

## State Machine

### Bond Lifecycle

```
                 ┌────────────────────┐
                 │   Bond Created     │
                 │  (issuance txn)    │
                 └────────────────────┘
                            │
                            ▼
         ┌──────────────────────────────────┐
         │                                  │
         │   Active (Pre-Maturity)          │
         │   - Holders can trade            │
         │   - Yield accumulates            │
         │   - Cannot redeem yet            │
         │                                  │
         └──────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
   ┌─────────┐         ┌──────────────────┐
   │Transfer │         │ Maturity Reached │
   │(2ndary) │         │(at ledger N)     │
   └─────────┘         └──────────────────┘
        │                       │
        │ Yield streams         │ Redemption now allowed
        │ update holders        │
        │                       ▼
        │              ┌──────────────────┐
        │              │   Redeemable     │
        └─────────────→│   - Holders      │
                       │     redeem       │
                       │   - Escrow       │
                       │     depletes     │
                       └──────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │  Fully Redeemed  │
                       │  (end of life)   │
                       └──────────────────┘
```

### Transaction Flow

#### Issue Bond

```
User (Issuer)
    │
    ├─ approve() USDC to contract
    │
    └─ issue_corporate_bond()
              ▼
    ┌─ Require auth
    ├─ Validate maturity > current_ledger
    ├─ Validate coupon_rate ∈ [0, 10000]
    ├─ Transfer escrow USDC: issuer → contract
    ├─ Store BondTerms
    ├─ Store EscrowRecord
    ├─ Set bondholder balance = face_value
    ├─ Initialize yield_stream
    ├─ Emit BondIssued event
    └─ Return success
```

#### Redeem Principal

```
User (Bondholder)
    │
    └─ redeem_principal()
         ▼
    ┌─ Require auth
    ├─ Load BondTerms
    ├─ Check current_ledger ≥ maturity_ledger
    ├─ Verify holder.balance ≥ amount
    ├─ Load EscrowRecord
    ├─ Check escrow.balance ≥ amount
    ├─ Burn tokens: holder.balance -= amount
    ├─ Transfer USDC: contract → holder
    ├─ Update escrow: escrow.balance -= amount
    ├─ Update total_redeemed += amount
    ├─ Remove yield_stream (if fully redeemed)
    ├─ Emit PrincipalRedeemed event
    └─ Return success
```

#### Transfer (Secondary Trading)

```
User (Bondholder A)
    │
    └─ transfer(A, B, bond_id, amount)
         ▼
    ┌─ Require auth on A
    ├─ Load BondTerms (verify bond exists)
    ├─ Check A.balance ≥ amount
    ├─ Transfer tokens: A.balance -= amount, B.balance += amount
    ├─ Load A's yield_stream
    ├─ Update A's yield stream (or remove if fully transferred)
    ├─ Create new yield_stream for B
    ├─ Register with Drips Network (Phase 2)
    ├─ Emit Transfer event
    └─ Return success
```

#### Query Accrued Interest

```
User (Analyst/Frontend)
    │
    └─ query_accrued_interest(bond_id, holder)
         ▼
    ┌─ Load BondTerms
    ├─ Load yield_stream for holder
    ├─ Calculate blocks_elapsed
    ├─ Calculate daily_coupon = face_value × coupon_rate_bps / 10000 / 365
    ├─ Calculate accrued = daily_coupon × blocks_elapsed / (365 × 75000)
    └─ Return accrued interest
```

## Mathematical Model

### Coupon Accrual Formula

**Annual Coupon (in base units):**
```
annual_coupon = face_value × (coupon_rate_bps / 10000)
```

Example: $100,000 face value @ 5% coupon
```
annual_coupon = 100,000,000,000 × (500 / 10000) = 5,000,000,000 (50 USDC)
```

**Daily Coupon:**
```
daily_coupon = annual_coupon / 365
```

**Accrued Interest (on any ledger):**
```
accrued_interest = daily_coupon × blocks_elapsed / blocks_per_year

where:
  blocks_per_year ≈ 365 × 75000 = 27,375,000 blocks
  blocks_elapsed = current_ledger - accrual_start_ledger
```

### Precision Guarantees

All calculations use **i128 fixed-point arithmetic**:

- **No floating-point**: Eliminates rounding errors
- **Base unit**: Smallest unit of stablecoin (1 microcent for USDC)
- **Range**: ±9.2 × 10^18 (sufficient for $billions in bonds)
- **Rounding**: Always floors (conservative, favors bondholders)

### Invariants Maintained

1. **Total Yield Invariant**
   ```
   sum(accrued_interest[all holders]) ≈ face_value × coupon_rate / 100 × years_elapsed
   ```

2. **Principal Conservation**
   ```
   sum(balances[all holders]) = total_minted - total_redeemed
   ```

3. **Maturity Finality**
   ```
   redemptions allowed ⟺ current_ledger ≥ maturity_ledger
   ```

4. **Escrow Sufficiency**
   ```
   escrow.balance ≥ sum(unredeemed_balances)
   ```

## Event Architecture

### Event Types & Topics

```rust
Topic: ("apex", "bond_issued")
├─ bond_id: Symbol
├─ issuer: Address
├─ face_value: i128
├─ coupon_rate_bps: u32
└─ maturity_ledger: u32

Topic: ("apex", "principal_redeemed")
├─ bond_id: Symbol
├─ holder: Address
├─ amount: i128
└─ timestamp: u32

Topic: ("apex", "transfer")
├─ from: Address
├─ to: Address
├─ bond_id: Symbol
└─ amount: i128
```

### Event Indexing

All events are published to Stellar event log and can be indexed by:

1. **Off-chain event indexers** (e.g., Stellar's RPC, Datonomy, etc.)
2. **Custom subgraph services** (e.g., TheGraph)
3. **Application-specific listeners** (direct RPC polling)

## Security Model

### Authorization

- **Soroban Native Auth**: All state changes require cryptographic signature
- **Per-operation**: `issuer.require_auth()`, `holder.require_auth()`
- **No role system**: Only transaction signer matters

### Reentrancy Protection

- **Atomic updates**: All storage modifications in single transaction
- **No external calls mid-state**: Escrow transfers happen at end
- **Drips integration**: Stateless, non-reentrant by design

### Asset Safety

- **Escrow lock**: Issuer cannot withdraw until maturity (hard ledger check)
- **Token burn**: Only contract can mint/burn
- **Balance consistency**: All transfers immediately update balances

## Integration Points

### Phase 1 (Current)
- **Stellar**: Native token transfers, authorization
- **Stablecoins**: USDC, EURC contracts (token::Client)
- **Events**: Stellar event log

### Phase 2 (Planned)
- **Drips Network**: Automatic yield distribution
- **Stellar AMMs**: Secondary market listing

### Phase 3+ (Future)
- **Cross-chain bridges**: Multi-network bonds
- **Portfolio services**: Analytics & aggregation
- **Rating agencies**: Credit score integration

## Performance Characteristics

### Gas Costs (Approximate)

| Operation | Cost (stroops) | Notes |
|-----------|---|---|
| Issue Bond | 5,000-10,000 | Storage write (BondTerms, Escrow, etc.) |
| Redeem Principal | 3,000-5,000 | Token burn + transfer |
| Transfer | 2,000-3,000 | Balance update + yield stream |
| Query | <100 | Read-only, no state change |

### Scalability

- **Bonds per contract**: Unlimited (1000s+)
- **Holders per bond**: Unlimited (1000s+)
- **Total TVL**: Limited by stablecoin cap, not contract

### Latency

- **Finality**: ~5 seconds (Stellar consensus)
- **Yield accrual**: Calculated at ledger sequence (block time ~5s)
- **Event indexing**: 10-30 seconds (off-chain indexers)

## Deployment Topology

### Recommended Architecture

```
┌──────────────────────────┐
│   Stellar Testnet        │
│  (Development/Testing)   │
│  Contract: 0xTEST...     │
└──────────────────────────┘
              │
              │ (audit & testing)
              ▼
┌──────────────────────────┐
│   Stellar Public Network │
│  (Production)            │
│  Contract: 0xPROD...     │
└──────────────────────────┘
              │
              │ (live bonds)
              ├─ Event indexer
              ├─ Frontend dApp
              ├─ Analytics service
              └─ Custodial bridges
```

---

**Status**: Production-Ready MVP (Phase 1)
**Last Updated**: June 19, 2026
