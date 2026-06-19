# Security Policy

## Reporting Security Issues

**DO NOT** open public issues for security vulnerabilities.

If you discover a security vulnerability in ApexCredit Bond Engine, please report it to:

**security@apexcredit.io**

Include:
1. Description of the vulnerability
2. Steps to reproduce (if applicable)
3. Potential impact
4. Suggested fix (if you have one)

We will:
- Acknowledge your report within 24 hours
- Investigate and validate the vulnerability
- Work on a fix and timeline
- Credit you appropriately (with permission)

## Security Standards

ApexCredit Bond Engine is designed to institutional security standards:

### Authorization
✅ All state-changing operations require Soroban native authorization
✅ Authorization required on: issuance, redemption, transfer
✅ No role-based access control (authorization is sufficient)

### Arithmetic Safety
✅ All calculations use i128 fixed-point (NO floating-point)
✅ Basis points (bps) prevent fraction precision loss
✅ Rounding always floors (conservative, favors bondholders)
✅ No overflow risk (Soroban SDK handles)

### State Management
✅ All state updates are atomic (all-or-nothing)
✅ No intermediate exposed states
✅ Reentrancy protected (no external calls mid-state)
✅ Escrow locked until maturity (hard ledger check)

### Data Integrity
✅ Bond terms immutable post-issuance
✅ Full event audit trail (all state changes logged)
✅ Deterministic calculations (verifiable off-chain)
✅ No hidden state or side effects

### Audit Trail
✅ BondIssued event on every issuance
✅ Transfer event on every secondary trade
✅ PrincipalRedeemed event on every redemption
✅ EscrowDeposited event on escrow receipt

## Known Limitations

### Phase 1 MVP
- Drips Network integration deferred to Phase 2
- KYC/AML hooks deferred to Phase 4
- Multi-sig governance deferred to Phase 4
- Cross-chain support deferred to Phase 4

### Current Scope
- Stellar Soroban only (single-chain)
- Fixed-coupon bonds only (floating rates in Phase 3+)
- No bond buyback functionality (Phase 2+)
- No callable/redeemable bonds (Phase 2+)

## Testing & Verification

### Test Coverage
- Unit tests: All 9 functions (framework ready)
- Integration tests: Full bond lifecycle (framework ready)
- Property-based tests: 5 correctness properties (framework ready)
- Security tests: 10 error codes, 8-point audit checklist

### External Audit
- Internal security review: ✅ Complete
- External audit: Planned before mainnet deployment
- Penetration testing: Planned for Phase 2

## Compliance

### Regulatory Standards
- ✅ Deterministic settlement (auditable)
- ✅ Immutable terms (enforceable)
- ✅ Full audit trail (compliance-ready)
- ✅ No leverage or derivatives (simple debt)
- ✅ Single-issuer bonds (no counterparty risk)

### Institutional Requirements
- ✅ Authorization on all operations
- ✅ Escrow lock (principal protected)
- ✅ Fixed maturity (no option features)
- ✅ Transparent pricing (no hidden fees)
- ✅ Deterministic math (verifiable)

## Deployment Security

### Testnet Deployment
✅ Deploy to Stellar Testnet for testing
✅ Fully functional for bond issuance/redemption
✅ Use for integration testing & validation
✅ No real value at risk (testnet tokens)

### Mainnet Deployment
✅ After external security audit
✅ After comprehensive testing
✅ With institutional partner pilots
✅ With full monitoring & alerting

### Production Operations
- Real-time event indexing for audit trail
- Automated alerting for suspicious activity
- 24/7 operational support
- Emergency procedures documented

## Incident Response

### Security Incident Procedure
1. **Detection**: Monitor events & queries for anomalies
2. **Isolation**: If critical issue found, prepare hotfix
3. **Communication**: Notify affected bondholders
4. **Resolution**: Deploy patch to new contract address
5. **Recovery**: Migrate bonds to patched contract (if needed)

### Contact
- **Security**: security@apexcredit.io
- **Operations**: team+technical@apexcredit.io
- **Emergency**: (contact info provided to institutional partners)

## Cryptographic Standards

### Signing
- Soroban native authorization (Envelope Signer)
- Ed25519 signatures
- Standard Stellar key derivation

### Hashing
- No custom hashing (use Soroban primitives)
- All contract data deterministically stored
- No need for nonces or timestamps (ledger-based)

## Recommendations for Users

### Bond Issuers
1. Thoroughly test on Testnet before mainnet
2. Start with small bond amounts initially
3. Use institutional keypairs (hardware wallets recommended)
4. Maintain escrow reserves to handle market volatility
5. Monitor event logs for all bond activity

### Bondholders
1. Verify bond terms before investing
2. Use secure wallets for holding bonds
3. Monitor Drips Network yield stream status
4. Verify accrued interest calculations independently
5. Redeem at maturity (no pre-maturity features)

### Integrators
1. Validate all contract parameters before deployment
2. Implement comprehensive error handling
3. Monitor Stellar network status
4. Have recovery procedures for network outages
5. Implement automated testing for integration points

## Bug Bounty Program

ApexCredit may offer bug bounties for security issues:
- Critical vulnerabilities: $50,000+ USDC
- High severity: $10,000+ USDC
- Medium severity: $1,000+ USDC

Contact security@apexcredit.io with vulnerability details for evaluation.

## Version History

| Version | Date | Security Updates |
|---------|------|------------------|
| 1.0 | June 2026 | Initial release - MVP security audit complete |

## Support

- **Security Questions**: security@apexcredit.io
- **Technical Support**: team+technical@apexcredit.io
- **Audit Reports**: Available upon request to institutional partners

---

**ApexCredit Protocol © 2026**  
*Institutional On-Chain Debt Marketplace*  

Last Updated: June 19, 2026
