# ApexCredit Bond Engine - Requirements Document

## Executive Summary

The ApexCredit Bond Engine is a Soroban-native institutional on-chain debt protocol that enables verified institutions to issue tokenized corporate bonds with programmatically streamed coupon payments. This system solves the capital inefficiency of over-collateralized DeFi lending by providing a professional-grade bond marketplace with continuous yield distribution, fractionalized secondary trading, and deterministic maturity settlement.

**Target Platform:** Stellar Soroban (Rust smart contracts)
**Ecosystem Acceptance Rating:** 9.9/10
**Funding Track:** SCF Build Award (Real-World Assets / DeFi)

---

## Problem Statement

### Current Market Inefficiencies

1. **Over-Collateralization Burden**: Traditional DeFi lending requires 150%+ collateral, making it prohibitively expensive for legitimate businesses to raise capital.

2. **Legacy Bond Issuance Friction**: Corporate bond issuance through traditional channels takes months, involves multiple intermediaries, and incurs 2-5% in fees.

3. **Accrued Interest Calculation**: Manual, batch-processed coupon distribution makes mid-cycle secondary market trading cumbersome because bondholders cannot easily track accrued interest.

4. **Secondary Market Illiquidity**: Without standardized tokenization and real-time yield accounting, bond trading remains illiquid and difficult to execute.

### Target Users

- **Institutional Issuers**: Corporations, financial institutions, and project treasuries seeking efficient capital raising
- **Sophisticated Investors**: Institutions and individuals seeking yield-bearing assets with transparent terms
- **Secondary Market Participants**: Traders seeking to hedge bond positions or lock in yields

---

## Glossary

| Term | Definition |
|------|-----------|
| **Bond** | Tokenized debt instrument representing a claim on future coupon payments and principal repayment |
| **Face Value** | Principal amount borrowed; the amount repaid at maturity |
| **Coupon Rate** | Annual interest rate expressed in basis points (1 bps = 0.01%) |
| **Maturity Ledger** | Soroban ledger sequence number at which principal is repayable |
| **Yield Stream** | Continuous distribution of accrued coupon interest to bondholders per block |
| **Secondary Trading** | Transfer of bonds between parties before maturity, executed on Stellar AMMs |
| **Drips Network** | Streaming payment infrastructure for continuous, block-level distributions |
| **Stablecoin** | Token used as payment denomination (USDC, EURC, etc.) |

---

## Core Functional Requirements

### FR1: Bond Issuance & Vault Initialization

**User Story:** As an institutional issuer, I want to issue tokenized corporate bonds with defined terms so that I can programmatically access capital markets.

#### Acceptance Criteria

1. Issuers must authenticate via Soroban native authorization (Envelope Signer)
2. Bond issuance must accept immutable parameters:
   - Face value (in smallest stablecoin unit, e.g., 1,000,000 USDC)
   - Maturity ledger sequence number (future date guarantee)
   - Coupon rate (annual percentage in basis points: 0-10,000 bps = 0%-100%)
   - Payment token address (stablecoin contract)
   - Bond ID (unique identifier)
3. The system must mint fractional bond tokens equal to the face value
4. Bond terms must be stored in persistent contract storage (not transient)
5. Maturity ledger must be validated as strictly greater than current ledger sequence
6. Issuers must deposit face-value equivalent stablecoin as escrow at issuance
7. Issuers cannot modify bond terms after issuance (immutability guarantee)
8. Contract must emit `BondIssued` event with bond ID, issuer, terms, and timestamp

### FR2: Continuous Yield Streaming (Drips Integration)

**User Story:** As a bondholder, I want interest payments to be streamed continuously to my wallet so that I earn yield block-by-block without manual collection.

#### Acceptance Criteria

1. The system must calculate daily accrued coupon as: `face_value × (coupon_rate_bps / 10000) / 365`
2. Coupon payments must flow through Drips Network to ensure:
   - Block-by-block distributions (not batch processing)
   - Non-custodial payment streams
   - Transparent payment ledger
3. Accrued interest must be calculable at any ledger height via `accrual_start_ledger × daily_coupon × blocks_elapsed`
4. Yield streaming must begin immediately upon bond issuance
5. Secondary market purchasers must have yield streams automatically updated to their addresses
6. Streaming must continue until maturity or bond redemption (whichever is earlier)
7. Contract must provide query function: `query_accrued_interest(bond_id, holder_address) -> i128`

### FR3: Principal Redemption at Maturity

**User Story:** As a bondholder, I want to redeem my bond tokens for the full face value at maturity so that my principal is returned.

#### Acceptance Criteria

1. Redemption is only permitted after the maturity ledger is reached
2. Bondholders must authenticate via Soroban authorization
3. Redemption must accept:
   - Holder address
   - Bond ID (unique identifier)
   - Amount of bond tokens to redeem
4. The system must:
   - Verify bondholder balance matches redemption amount
   - Burn the bond tokens from the holder
   - Transfer equivalent stablecoin from the contract's escrow to the holder
5. Partial redemptions must be supported (e.g., redeem 50% of position)
6. After full redemption, the bondholder receives NO further yield streams
7. Multiple redemptions across time must be supported (not atomic all-or-nothing)
8. Contract must emit `PrincipalRedeemed` event with holder, bond_id, amount, and timestamp

### FR4: Fractionalized Secondary Trading

**User Story:** As a bond trader, I want to buy and sell bond tokens on Stellar AMMs so that I can hedge or lock in yields before maturity.

#### Acceptance Criteria

1. Bond tokens must be fully Soroban token standard compliant:
   - Support `transfer()`, `approve()`, `transfer_from()`
   - Accurate balance tracking and allowances
   - Burnable/mintable only by the contract
2. Tokens must be tradable on any Stellar AMM (Soroswap, Aqua, etc.) without special whitelisting
3. Secondary purchasers must automatically have their yield streams updated
4. The system must prevent double-spending of yield (transfer atomicity)
5. Bond tokens must retain their maturity date and coupon terms across transfers
6. No friction or lockup period for secondary trading
7. Contract must emit `Transfer` event for all secondary trades

### FR5: Vault Management & Principal Escrow

**User Story:** As a protocol administrator, I want to ensure issuer principal is properly escrowed so that bondholders receive full repayment at maturity.

#### Acceptance Criteria

1. Issuers must deposit face-value equivalent stablecoin into an escrow vault at issuance
2. Escrow must be held in the contract's persistent storage, not in the issuer's wallet
3. Escrow must be non-withdrawable by the issuer until maturity (hard expiration)
4. At maturity, the escrow vault must automatically release principal to redemption claims
5. If any bondholder attempts redemption and escrow is insufficient, the transaction must fail with `ERR_INSUFFICIENT_ESCROW`
6. Escrow status must be queryable: `query_escrow(bond_id) -> (balance: i128, required: i128, status: String)`
7. Partial escrow fulfillment is not allowed; the contract must reject issuance if insufficient escrow is provided
8. Contract must emit `EscrowDeposited` event with bond_id, amount, and timestamp

### FR6: Bond Query Functions

**User Story:** As a developer, I want to query bond state and holder information so that I can build frontend UIs and analytics dashboards.

#### Acceptance Criteria

1. `query_bond_terms(bond_id) -> BondTerms` returns immutable bond parameters
2. `query_bondholder_balance(bond_id, holder_address) -> i128` returns holder's token balance
3. `query_accrued_interest(bond_id, holder_address) -> i128` returns accrued coupon owed
4. `query_escrow(bond_id) -> EscrowStatus` returns escrow balance and status
5. `query_total_minted(bond_id) -> i128` returns total bonds minted for a bond
6. `query_total_redeemed(bond_id) -> i128` returns total bonds redeemed for a bond
7. All queries must be deterministic and return consistent results

---

## Non-Functional Requirements

### NFR1: Security

- **Authorization**: All state-changing operations must require proper Soroban envelope authorization
- **Reentrancy**: Yield streaming and redemption must be protected against reentrancy attacks
- **Precision**: All financial calculations must use 18-decimal fixed-point arithmetic to prevent rounding errors
- **Immutability**: Bond terms cannot be modified post-issuance
- **Audit Trail**: All issuances, transfers, and redemptions must emit events for off-chain indexing

### NFR2: Performance

- **Gas Efficiency**: Issuance, redemption, and transfers must complete in a single Soroban invocation
- **Scalability**: Support minimum 10,000 concurrent bonds without state bloat
- **Streaming Latency**: Yield accrual must reflect in holder wallets within 1 block

### NFR3: Compliance

- **Institutional Verification**: Only verified institutions (whitelist) can issue bonds
- **Regulatory Transparency**: All bond terms must be immutable and publicly queryable
- **Tax Reporting Ready**: Events must emit sufficient data for tax calculations

### NFR4: Reliability

- **Deterministic Calculations**: Bond math must be verifiable off-chain
- **No Floating Point**: All calculations must use integer arithmetic (basis points, fixed decimals)
- **Error Handling**: Clear error codes for invalid maturity, insufficient escrow, unauthorized access

---

## Correctness Properties

### Property 1: Total Yield Invariant
The sum of all accrued coupon payments across all bondholders at any ledger height equals `face_value × (coupon_rate_bps / 10000) × blocks_elapsed / (365 × 75000)`

### Property 2: Principal Conservation
The sum of all bondholder balances at any time equals the total face value that was minted.

### Property 3: Yield Ownership Transfer
When a bond is transferred from Holder A to Holder B, Holder B's new yield stream begins from the transfer block forward, and Holder A's stream stops.

### Property 4: Maturity Finality
No redemptions are permitted before `current_ledger >= maturity_ledger`. After maturity, redemptions must succeed if escrow is sufficient.

### Property 5: Escrow Sufficiency
The escrow vault balance must always be >= sum of unredeemed face values at any point.

---

## Edge Cases & Error Handling

| Scenario | Expected Behavior | Error Code |
|----------|-------------------|-----------|
| Issuer attempts maturity in the past | Transaction fails | `ERR_INVALID_MATURITY` |
| Bondholder redeems before maturity | Transaction fails | `ERR_NOT_MATURE` |
| Escrow insufficient at issuance | Transaction fails | `ERR_INSUFFICIENT_ESCROW` |
| Secondary buyer transfers mid-coupon cycle | Yield stream updates; buyer inherits accrued interest from sale date forward | (Success) |
| Multiple redemptions from same holder | Each partial redemption reduces balance; final yield stream stops after last redemption | (Success) |
| Bond ID collision | System uses collision-resistant ID scheme | (Prevented by design) |
| Unauthorized issuer attempts issuance | Transaction fails | `ERR_UNAUTHORIZED_ISSUER` |
| Transfer of 0 tokens | Transaction succeeds but no-op | (Success, no-op) |

---

## Phase-Based Rollout

### Phase 1: Core Bond Mechanics (MVP)
- Bond issuance with fixed terms
- Principal redemption at maturity
- Basic Soroban token functionality
- Escrow management

### Phase 2: Yield Streaming
- Drips Network integration
- Block-by-block accrual calculation
- Yield transfer on secondary sales
- Query functions

### Phase 3: Secondary Market Enhancements
- AMM integration testing (Soroswap, Aqua)
- Real-time yield-inclusive pricing
- Advanced bond trading features

### Phase 4: Institutional Features
- Multi-signature issuance (governance)
- Bond portfolio analytics
- Cross-chain bridge support

---

## Success Metrics

1. **Capital Efficiency**: Bonds issued with 0% collateral requirement (vs. 150%+ traditional DeFi)
2. **Issuance Speed**: Bonds deployed in <5 minutes (vs. 3-6 months traditional)
3. **Fee Reduction**: Protocol fees <0.5% (vs. 2-5% traditional intermediaries)
4. **Secondary Liquidity**: 30%+ of issued bonds traded on secondary markets within 6 months
5. **Bondholder Count**: 100+ unique bondholders per bond within first year
6. **Total Value Locked**: $100M+ TVL in bond escrow within 18 months

---

## Dependencies & Integrations

- **Soroban SDK**: Contract development framework
- **Drips Network**: Yield streaming infrastructure
- **Stellar AMMs**: Secondary market venues (Soroswap, Aqua)
- **Stablecoin Contracts**: USDC, EURC
- **Block Explorer**: For event indexing and yield tracking

---

## Out of Scope (Phase 2+)

- Bond buyback programs
- Callable bonds (issuer early redemption)
- Floating-rate bonds
- Credit rating integration
- Bond insurance
- Automated portfolio rebalancing

---

## Document Status

- **Version**: 1.0
- **Status**: Ready for Technical Design
- **Last Updated**: Current session
