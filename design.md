# ApexCredit Bond Engine - Technical Design Document

## Architecture Overview

The ApexCredit Bond Engine is built on Soroban as a core contract that manages bond issuance, escrow, and yield streaming integration. The system is designed for institutional capital markets with emphasis on security, precision, and auditability.

### System Components

```
┌─────────────────────────────────────────────────────────┐
│                    Soroban Contract Layer                 │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────────┐    ┌──────────────────┐           │
│  │  Bond Registry   │    │  Escrow Manager  │           │
│  │  (Persistent)    │    │  (Persistent)    │           │
│  └──────────────────┘    └──────────────────┘           │
│           │                        │                     │
│  ┌──────────────────────────────────────────┐           │
│  │   ApexCreditEngine (Main Contract)       │           │
│  │                                           │           │
│  │  - issue_corporate_bond()                │           │
│  │  - redeem_principal()                    │           │
│  │  - transfer() [token ops]                │           │
│  │  - approve() / transfer_from()           │           │
│  │  - query_* functions                     │           │
│  └──────────────────────────────────────────┘           │
│           │                                              │
│  ┌──────────────────────────────────────────┐           │
│  │   Yield Streaming Module                  │           │
│  │  (Drips Network Integration)              │           │
│  └──────────────────────────────────────────┘           │
│                                                           │
└─────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
    ┌─────────┐          ┌─────────┐         ┌────────┐
    │ Drips   │          │ Stellar │         │ Events │
    │ Network │          │ AMMs    │         │ Log    │
    └─────────┘          └─────────┘         └────────┘
```

---

## Data Structures

### 1. BondTerms (Persistent Storage)

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondTerms {
    pub issuer: Address,
    pub bond_id: Symbol,
    pub face_value: i128,           // In stablecoin smallest unit (e.g., USDC)
    pub maturity_ledger: u32,       // Ledger sequence for maturity
    pub coupon_rate_bps: u32,       // Annual rate in basis points (0-10000)
    pub payment_token: Address,     // Stablecoin contract address
    pub issued_ledger: u32,         // Ledger when bond was issued
}
```

**Storage Key Pattern**: `bond_terms_{bond_id}`

### 2. Escrow Storage

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub bond_id: Symbol,
    pub issuer: Address,
    pub escrow_balance: i128,       // Current stablecoin held
    pub required_balance: i128,     // Face value (immutable)
    pub created_ledger: u32,
    pub maturity_ledger: u32,
}
```

**Storage Key Pattern**: `escrow_{bond_id}`

### 3. Bondholder Balances (Token Ledger)

```rust
// Standard Soroban token ledger
// Key pattern: balance_{bond_id}_{holder_address}
// Type: i128
```

### 4. Yield Stream State

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YieldStream {
    pub bond_id: Symbol,
    pub holder: Address,
    pub accrual_start_ledger: u32,  // When yield accrual began for this holder
    pub last_collection_ledger: u32, // Last block yield was streamed
    pub drips_stream_id: u64,       // Reference to Drips Network stream
}
```

**Storage Key Pattern**: `yield_stream_{bond_id}_{holder_address}`

### 5. Event Types

```rust
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ContractEvent {
    BondIssued {
        bond_id: Symbol,
        issuer: Address,
        face_value: i128,
        coupon_rate_bps: u32,
        maturity_ledger: u32,
    },
    EscrowDeposited {
        bond_id: Symbol,
        issuer: Address,
        amount: i128,
        timestamp: u32,
    },
    Transfer {
        from: Address,
        to: Address,
        bond_id: Symbol,
        amount: i128,
    },
    YieldStreamed {
        bond_id: Symbol,
        holder: Address,
        amount: i128,
        timestamp: u32,
    },
    PrincipalRedeemed {
        bond_id: Symbol,
        holder: Address,
        amount: i128,
        timestamp: u32,
    },
}
```

---

## Core Functions

### 1. `issue_corporate_bond(env, issuer, bond_id, terms, escrow_amount)`

**Purpose**: Issue a new bond with immutable terms and escrow deposit.

**Preconditions**:
- `issuer` is authorized (Soroban native auth)
- `terms.maturity_ledger > env.ledger().sequence()` (future maturity)
- `0 < terms.coupon_rate_bps <= 10000` (valid rate)
- `escrow_amount >= terms.face_value` (sufficient escrow)

**Algorithm**:
```
1. Verify issuer authorization
2. Validate maturity > current ledger
3. Validate coupon rate in range
4. Check escrow deposit >= face value
5. Mint bond tokens to issuer (amount = face_value)
6. Transfer escrow stablecoin from issuer to contract
7. Store BondTerms in persistent storage
8. Store EscrowRecord in persistent storage
9. Initialize YieldStream for issuer
10. Register yield stream with Drips Network
11. Emit BondIssued event
12. Return bond_id
```

**Error Codes**:
- `ERR_UNAUTHORIZED`: Issuer failed authorization
- `ERR_INVALID_MATURITY`: Maturity in past
- `ERR_INVALID_COUPON_RATE`: Rate outside valid range
- `ERR_INSUFFICIENT_ESCROW`: Escrow < face value
- `ERR_DUPLICATE_BOND_ID`: Bond ID already exists

---

### 2. `redeem_principal(env, holder, bond_id, amount)`

**Purpose**: Redeem bond tokens for stablecoin principal at or after maturity.

**Preconditions**:
- `holder` is authorized
- `env.ledger().sequence() >= maturity_ledger`
- `holder` balance >= `amount`
- `escrow` balance >= `amount`

**Algorithm**:
```
1. Verify holder authorization
2. Retrieve BondTerms for bond_id
3. Assert current_ledger >= maturity_ledger
4. Verify holder has sufficient balance
5. Retrieve EscrowRecord
6. Assert escrow balance >= amount
7. Burn bond tokens from holder
8. Transfer stablecoin from escrow to holder
9. Update escrow balance
10. Update yield stream (stop accrual if full redemption)
11. Emit PrincipalRedeemed event
12. Return success
```

**Error Codes**:
- `ERR_UNAUTHORIZED`: Holder failed authorization
- `ERR_NOT_MATURE`: Maturity not reached
- `ERR_INSUFFICIENT_BALANCE`: Holder has fewer tokens
- `ERR_INSUFFICIENT_ESCROW`: Escrow insufficient
- `ERR_BOND_NOT_FOUND`: Bond ID not found

---

### 3. `transfer(env, from, to, bond_id, amount)` [Token Transfer]

**Purpose**: Transfer bond tokens between holders (secondary trading).

**Preconditions**:
- `from` is authorized
- `from` balance >= `amount`
- `to` is a valid address

**Algorithm**:
```
1. Verify from authorization
2. Retrieve bond balance ledger
3. Assert from balance >= amount
4. Subtract amount from from balance
5. Add amount to to balance
6. Retrieve current yield stream for from
7. Calculate accrued interest for from (from issue or last transfer)
8. Stop yield stream for from (if fully transferred)
9. Create new yield stream for to starting current ledger
10. Register new stream with Drips Network
11. Emit Transfer event
12. Return success
```

**Error Codes**:
- `ERR_UNAUTHORIZED`: From failed authorization
- `ERR_INSUFFICIENT_BALANCE`: From has fewer tokens
- `ERR_INVALID_RECIPIENT`: To address invalid

---

### 4. `query_accrued_interest(env, bond_id, holder_address) -> i128`

**Purpose**: Calculate accrued coupon interest for a holder at current ledger.

**Formula**:
```
accrued_interest = face_value 
                 × (coupon_rate_bps / 10000) 
                 × (current_ledger - accrual_start_ledger) 
                 / (365 × 75000)  // 75000 blocks per year on Stellar
```

**Implementation**:
```
1. Retrieve BondTerms for bond_id
2. Retrieve YieldStream for holder
3. Get current ledger sequence
4. Calculate blocks_elapsed = current_ledger - accrual_start_ledger
5. Calculate annual_coupon = face_value × coupon_rate_bps / 10000
6. Calculate daily_coupon = annual_coupon / 365
7. Calculate block_coupon = daily_coupon / (75000 / 365)
8. Return blocks_elapsed × block_coupon
```

**Precision Note**: All arithmetic uses i128 fixed-point (no decimals, scaled by token precision).

---

### 5. `query_bond_terms(env, bond_id) -> BondTerms`

**Purpose**: Retrieve immutable bond parameters.

**Implementation**:
```
1. Look up bond_id in persistent storage
2. Return BondTerms if found
3. Return error if not found
```

---

### 6. `query_escrow(env, bond_id) -> (balance, required, status)`

**Purpose**: Return escrow status.

**Implementation**:
```
1. Retrieve EscrowRecord for bond_id
2. If current_ledger > maturity_ledger, status = "MATURED"
3. Else if balance < required, status = "UNDERFUNDED"
4. Else status = "ACTIVE"
5. Return (balance, required, status)
```

---

## Yield Streaming Integration (Drips Network)

### Flow Overview

1. **At Issuance**:
   - Issuer receives bond tokens
   - YieldStream created for issuer with `accrual_start_ledger = current_ledger`
   - Drips stream registered to issuer for daily coupon accrual

2. **On Secondary Transfer**:
   - Buyer receives bond tokens
   - Seller's yield stream stops (accrual finalized)
   - Buyer's new yield stream starts at transfer ledger
   - Drips stream updated to reflect new recipient

3. **Continuous Accrual**:
   - Daily coupon calculated as: `face_value × coupon_rate_bps / 10000 / 365`
   - Drips Network distributes this daily to current holder
   - Accrual is deterministic and auditable

### Drips Integration API

```rust
pub fn register_yield_stream(
    env: &Env,
    bond_id: Symbol,
    holder: Address,
    daily_coupon: i128,
) -> u64 {
    // Call Drips Network contract to register streaming payment
    // Return stream_id for tracking
}

pub fn update_yield_stream_recipient(
    env: &Env,
    stream_id: u64,
    new_recipient: Address,
) {
    // Call Drips Network to update recipient
}

pub fn stop_yield_stream(
    env: &Env,
    stream_id: u64,
) {
    // Call Drips Network to terminate stream
}
```

---

## Security Considerations

### 1. Authorization

All state-changing functions require Soroban native authorization:
```rust
issuer.require_auth();
holder.require_auth();
```

### 2. Reentrancy Protection

Transfer operations are protected by atomic state updates:
- Balance ledger updates happen in single transaction
- No intermediate states exposed to external calls
- Drips Network calls are non-reentrant by design

### 3. Precision & Rounding

- All values stored as i128 (no floating-point)
- Basis points (bps) prevent fraction precision loss
- Daily coupon calculated once per block per bond
- Rounding always floors to conservative side (bondholder benefits)

### 4. Escrow Lock

- Escrow stored in contract, not issuer's wallet
- Issuer cannot withdraw until after maturity
- Hard ledger-based expiration enforced

### 5. Token Standard Compliance

- All transfers follow Soroban token interface
- Allowances (approve/transfer_from) fully supported
- Burn operations only by contract

---

## Gas Optimization Strategies

### 1. Batch Operations

- Single invocation for multiple transfers (if supported by AMM)
- Yield accrual calculated once per holder per query (lazy evaluation)

### 2. Storage Patterns

- Use Symbol for bond_id (cheaper than String)
- Pack related fields in single struct where possible
- Lazy-load escrow records only when needed

### 3. Drips Network Calls

- Register yield streams once at issuance
- Update only on secondary transfer
- Avoid redundant stream creation

---

## Testing Strategy

### Unit Tests

1. **Issuance Tests**:
   - Valid bond creation with valid parameters
   - Rejection of past maturity dates
   - Rejection of invalid coupon rates
   - Escrow validation

2. **Redemption Tests**:
   - Redemption allowed only after maturity
   - Partial vs. full redemptions
   - Escrow depletion checks

3. **Transfer Tests**:
   - Secondary market transfers
   - Yield stream ownership updates
   - Balance ledger consistency

4. **Yield Calculation Tests**:
   - Accrual formula correctness
   - Precision edge cases
   - Multiple holders on same bond

### Property-Based Tests

1. **Total Yield Invariant**: Sum of all accrued interest == expected total
2. **Principal Conservation**: Sum of balances == total minted
3. **Yield Ownership**: No double-accrual on transfers
4. **Maturity Finality**: No pre-maturity redemptions allowed
5. **Escrow Sufficiency**: Escrow always >= unredeemed balances

### Integration Tests

1. Bond issuance → secondary transfer → redemption flow
2. Multiple concurrent bonds with different terms
3. Drips Network stream lifecycle
4. AMM integration (token swaps)

---

## Error Codes & Messages

```rust
const ERR_UNAUTHORIZED: &str = "401";
const ERR_INVALID_MATURITY: &str = "402";
const ERR_INVALID_COUPON_RATE: &str = "403";
const ERR_INSUFFICIENT_ESCROW: &str = "404";
const ERR_DUPLICATE_BOND_ID: &str = "405";
const ERR_NOT_MATURE: &str = "406";
const ERR_INSUFFICIENT_BALANCE: &str = "407";
const ERR_BOND_NOT_FOUND: &str = "408";
const ERR_INVALID_RECIPIENT: &str = "409";
const ERR_ESCROW_UNDERFUNDED: &str = "410";
```

---

## Deployment Architecture

### Contract Structure

```
contracts/
└── apex_core/
    ├── Cargo.toml
    └── src/
        ├── lib.rs              # Main contract entry
        ├── bond_math.rs        # Coupon calculations
        ├── stream_router.rs    # Drips integration
        ├── escrow.rs           # Escrow management
        ├── events.rs           # Event definitions
        └── tests/
            ├── unit_tests.rs
            ├── property_tests.rs
            └── integration_tests.rs
```

### Soroban Build & Deploy

- **Network**: Stellar Testnet (initial), Stellar Public (production)
- **Build**: `soroban contract build`
- **Deploy**: `soroban contract deploy --source <key> --network testnet`
- **Verify**: `soroban contract info --id <contract_id>`

---

## Phase Milestones

### Phase 1: MVP (Core Bond Mechanics)
- [x] BondTerms storage
- [x] Issue bond (no escrow validation)
- [x] Redemption at maturity
- [x] Basic token transfer
- [ ] Query functions

### Phase 2: Escrow & Yield (Full Requirements)
- [ ] Escrow deposit validation
- [ ] Escrow lock mechanism
- [ ] Drips Network integration
- [ ] Yield accrual calculation
- [ ] Yield stream transfer on secondary sales

### Phase 3: Hardening & Testing
- [ ] Unit test coverage 90%+
- [ ] Property-based test coverage
- [ ] Integration test coverage
- [ ] Audit preparation

### Phase 4: Production
- [ ] Security audit (3rd-party)
- [ ] Mainnet deployment
- [ ] Monitoring & alerting

---

## Document Status

- **Version**: 1.0
- **Status**: Ready for Implementation
- **Last Updated**: Current session
