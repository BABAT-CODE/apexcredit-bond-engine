# Contributing to ApexCredit Bond Engine

ApexCredit Bond Engine is an institutional-grade Soroban protocol for tokenized bonds with Drips Network yield streaming. We welcome contributions from developers, researchers, and institutional partners.

## Development Workflow

### Prerequisites

- Rust 1.75+ ([Install](https://rustup.rs/))
- Soroban CLI ([Install](https://soroban.stellar.org/docs/getting-started/setup))
- WASM target: `rustup target add wasm32-unknown-unknown`

### Build & Test

```bash
cd contracts/apex_core

# Build
cargo build --target wasm32-unknown-unknown --release

# Test
cargo test

# Lint & Format
cargo fmt --check
cargo clippy
```

### Deployment (Testnet)

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/apex_credit.wasm \
  --source <keypair> \
  --network testnet
```

See [DEPLOYMENT.md](./DEPLOYMENT.md) for full instructions.

## Code Quality Standards

### Authorization
- All state-changing functions must require authorization
- Use `issuer.require_auth()`, `holder.require_auth()`
- Document authorization requirements in function comments

### Arithmetic
- Use i128 fixed-point only (NO floating-point)
- Basis points (bps) for percentages (0-10000 = 0%-100%)
- Round conservatively (floor toward bondholder benefit)

### Storage
- Use descriptive key patterns: `"key_name_{id}_{optional_field}"`
- Store immutable data in persistent storage
- Document storage lifetime and mutability

### Error Handling
- Use defined error codes (401-410)
- Include context in error messages
- Document error conditions in function comments

### Events
- Emit events for all state changes
- Include relevant metadata in events
- Use consistent topic naming: `("apex", "event_name")`

## Testing Requirements

### Unit Tests
- Test all happy paths
- Test all error conditions
- Include edge cases (zero values, max i128, etc.)
- Minimum 90% coverage for new code

### Integration Tests
- Test cross-function interactions
- Test full bond lifecycle: issuance → transfer → redemption
- Test escrow lock behavior
- Test yield stream updates on transfer

### Property-Based Tests
- Test correctness properties:
  1. Total Yield Invariant
  2. Principal Conservation
  3. Yield Ownership Transfer
  4. Maturity Finality
  5. Escrow Sufficiency
- Run with 1000+ generated test cases

## Security Considerations

### Before Submitting Code

- [ ] All state changes require authorization
- [ ] No floating-point arithmetic (i128 only)
- [ ] Bond terms cannot be modified post-issuance
- [ ] Escrow cannot be withdrawn before maturity
- [ ] No reentrancy vulnerabilities
- [ ] All storage updates are atomic

### Security Review Process

1. **Internal Review**: Code review by core team
2. **Test Coverage**: Verify 90%+ test coverage
3. **Static Analysis**: Run `cargo clippy`
4. **External Audit**: Scheduled before mainnet deployment

## Documentation Standards

### Function Comments

```rust
/// Brief one-line description.
///
/// Longer explanation of purpose, parameters, and return value.
///
/// # Authorization
/// Requires `holder.require_auth()`.
///
/// # Errors
/// Returns `ERR_NOT_MATURE` if maturity ledger not reached.
/// Returns `ERR_INSUFFICIENT_BALANCE` if holder has fewer tokens.
///
/// # Example
/// ```
/// redeem_principal(env, holder, bond_id, amount)
/// ```
pub fn redeem_principal(env: Env, holder: Address, bond_id: Symbol, amount: i128) {
    // ...
}
```

### Changelog Format

```markdown
## [Version] - YYYY-MM-DD

### Added
- New features

### Changed
- Modified behavior

### Fixed
- Bug fixes

### Security
- Security-related changes
```

## Roadmap

### Phase 1: Core Bond Mechanics (✅ Complete)
- Bond issuance with escrow
- Principal redemption at maturity
- Secondary trading
- Query functions

### Phase 2: Drips Integration (4-6 weeks)
- Yield stream registration at issuance
- Update Drips recipient on transfer
- Stop streams on redemption
- Implement `register_yield_stream()`, `update_recipient()`, `stop_stream()`

### Phase 3: Secondary Markets (2-3 months)
- Soroswap/Aqua AMM integration
- Yield-aware pricing
- Portfolio analytics

### Phase 4: Institutional Features (3-4 months)
- Multi-sig governance
- KYC/AML provider hooks
- Cross-chain bridge support

## Getting Help

### Documentation
- [README.md](./README.md) - Quick start & API reference
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design & security model
- [design.md](./design.md) - Technical specifications
- [requirements.md](./requirements.md) - Business requirements

### Communication
- **Technical Questions**: team+technical@apexcredit.io
- **Security Issues**: security@apexcredit.io
- **Partnerships**: team+partnerships@apexcredit.io

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. All contributors must:

- Be respectful and professional
- Provide constructive feedback
- Focus on the problem, not the person
- Report security issues privately

## License

ApexCredit Protocol © 2026. Proprietary.

---

**Questions?** Open an issue or reach out to team+technical@apexcredit.io
