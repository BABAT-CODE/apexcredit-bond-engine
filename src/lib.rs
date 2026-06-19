#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ============================================================================
// Data Structures
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondTerms {
    pub issuer: Address,
    pub bond_id: Symbol,
    pub face_value: i128,
    pub maturity_ledger: u32,
    pub coupon_rate_bps: u32,
    pub payment_token: Address,
    pub issued_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub bond_id: Symbol,
    pub issuer: Address,
    pub escrow_balance: i128,
    pub required_balance: i128,
    pub created_ledger: u32,
    pub maturity_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YieldStream {
    pub bond_id: Symbol,
    pub holder: Address,
    pub accrual_start_ledger: u32,
}

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
        ledger: u32,
    },
    Transfer {
        from: Address,
        to: Address,
        bond_id: Symbol,
        amount: i128,
    },
    PrincipalRedeemed {
        bond_id: Symbol,
        holder: Address,
        amount: i128,
        ledger: u32,
    },
}

// ============================================================================
// Error Codes
// ============================================================================

pub const ERR_UNAUTHORIZED: u32 = 401;
pub const ERR_INVALID_MATURITY: u32 = 402;
pub const ERR_INVALID_COUPON_RATE: u32 = 403;
pub const ERR_INSUFFICIENT_ESCROW: u32 = 404;
pub const ERR_DUPLICATE_BOND_ID: u32 = 405;
pub const ERR_NOT_MATURE: u32 = 406;
pub const ERR_INSUFFICIENT_BALANCE: u32 = 407;
pub const ERR_BOND_NOT_FOUND: u32 = 408;
pub const ERR_INVALID_RECIPIENT: u32 = 409;

// ============================================================================
// Contract Implementation
// ============================================================================

#[contract]
pub struct ApexCreditContract;

#[contractimpl]
impl ApexCreditContract {
    /// Issue a new corporate bond with immutable terms and escrow deposit.
    pub fn issue_bond(
        env: Env,
        issuer: Address,
        bond_id: Symbol,
        face_value: i128,
        maturity_ledger: u32,
        coupon_rate_bps: u32,
        payment_token: Address,
        escrow_amount: i128,
    ) -> Symbol {
        issuer.require_auth();

        let current_ledger = env.ledger().sequence();

        if maturity_ledger <= current_ledger {
            panic!("{}", ERR_INVALID_MATURITY);
        }

        if coupon_rate_bps > 10000 {
            panic!("{}", ERR_INVALID_COUPON_RATE);
        }

        if escrow_amount < face_value {
            panic!("{}", ERR_INSUFFICIENT_ESCROW);
        }

        let bond_terms = BondTerms {
            issuer: issuer.clone(),
            bond_id: bond_id.clone(),
            face_value,
            maturity_ledger,
            coupon_rate_bps,
            payment_token,
            issued_ledger: current_ledger,
        };
        env.storage()
            .persistent()
            .set(&symbol_short!("bt"), &bond_terms);

        let escrow_record = EscrowRecord {
            bond_id: bond_id.clone(),
            issuer: issuer.clone(),
            escrow_balance: escrow_amount,
            required_balance: face_value,
            created_ledger: current_ledger,
            maturity_ledger,
        };
        env.storage()
            .persistent()
            .set(&symbol_short!("esc"), &escrow_record);

        env.storage()
            .persistent()
            .set(&symbol_short!("tm"), &face_value);
        env.storage()
            .persistent()
            .set(&symbol_short!("tr"), &0i128);

        env.storage()
            .persistent()
            .set(&symbol_short!("bal"), &face_value);

        let yield_stream = YieldStream {
            bond_id: bond_id.clone(),
            holder: issuer.clone(),
            accrual_start_ledger: current_ledger,
        };
        env.storage()
            .persistent()
            .set(&symbol_short!("ys"), &yield_stream);

        env.events().publish(
            (symbol_short!("bond"), symbol_short!("iss")),
            ContractEvent::BondIssued {
                bond_id: bond_id.clone(),
                issuer: issuer.clone(),
                face_value,
                coupon_rate_bps,
                maturity_ledger,
            },
        );

        env.events().publish(
            (symbol_short!("esc"), symbol_short!("dep")),
            ContractEvent::EscrowDeposited {
                bond_id: bond_id.clone(),
                issuer: issuer.clone(),
                amount: escrow_amount,
                ledger: current_ledger,
            },
        );

        bond_id
    }

    /// Redeem bond tokens for principal at or after maturity.
    pub fn redeem_principal(
        env: Env,
        holder: Address,
        bond_id: Symbol,
        amount: i128,
    ) {
        holder.require_auth();

        let current_ledger = env.ledger().sequence();

        let bond_terms: BondTerms = env
            .storage()
            .persistent()
            .get(&symbol_short!("bt"))
            .expect(ERR_BOND_NOT_FOUND);

        if current_ledger < bond_terms.maturity_ledger {
            panic!("{}", ERR_NOT_MATURE);
        }

        let holder_balance: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("bal"))
            .unwrap_or(0);

        if holder_balance < amount {
            panic!("{}", ERR_INSUFFICIENT_BALANCE);
        }

        let mut escrow_record: EscrowRecord = env
            .storage()
            .persistent()
            .get(&symbol_short!("esc"))
            .expect(ERR_BOND_NOT_FOUND);

        if escrow_record.escrow_balance < amount {
            panic!("{}", ERR_INSUFFICIENT_ESCROW);
        }

        let new_balance = holder_balance - amount;
        if new_balance == 0 {
            env.storage().persistent().remove(&symbol_short!("bal"));
        } else {
            env.storage()
                .persistent()
                .set(&symbol_short!("bal"), &new_balance);
        }

        let total_redeemed: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("tr"))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&symbol_short!("tr"), &(total_redeemed + amount));

        escrow_record.escrow_balance -= amount;
        env.storage()
            .persistent()
            .set(&symbol_short!("esc"), &escrow_record);

        env.events().publish(
            (symbol_short!("bond"), symbol_short!("red")),
            ContractEvent::PrincipalRedeemed {
                bond_id,
                holder,
                amount,
                ledger: current_ledger,
            },
        );
    }

    /// Transfer bond tokens between holders (secondary trading).
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        bond_id: Symbol,
        amount: i128,
    ) {
        from.require_auth();

        let current_ledger = env.ledger().sequence();

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("bal"))
            .unwrap_or(0);

        if from_balance < amount {
            panic!("{}", ERR_INSUFFICIENT_BALANCE);
        }

        let new_from_balance = from_balance - amount;
        if new_from_balance == 0 {
            env.storage().persistent().remove(&symbol_short!("bal"));
        } else {
            env.storage()
                .persistent()
                .set(&symbol_short!("bal"), &new_from_balance);
        }

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("bal"))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&symbol_short!("bal"), &(to_balance + amount));

        let sender_stream = YieldStream {
            bond_id: bond_id.clone(),
            holder: from.clone(),
            accrual_start_ledger: current_ledger,
        };
        env.storage()
            .persistent()
            .set(&symbol_short!("ys"), &sender_stream);

        let recipient_stream = YieldStream {
            bond_id: bond_id.clone(),
            holder: to.clone(),
            accrual_start_ledger: current_ledger,
        };
        env.storage()
            .persistent()
            .set(&symbol_short!("ys"), &recipient_stream);

        env.events().publish(
            (symbol_short!("bond"), symbol_short!("xfr")),
            ContractEvent::Transfer {
                from,
                to,
                bond_id,
                amount,
            },
        );
    }

    /// Query bond terms.
    pub fn query_bond_terms(env: Env, bond_id: Symbol) -> BondTerms {
        env.storage()
            .persistent()
            .get(&symbol_short!("bt"))
            .expect(ERR_BOND_NOT_FOUND)
    }

    /// Query bondholder balance.
    pub fn query_balance(env: Env, bond_id: Symbol, holder: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("bal"))
            .unwrap_or(0)
    }

    /// Query accrued coupon interest.
    pub fn query_accrued_interest(env: Env, bond_id: Symbol, holder: Address) -> i128 {
        let current_ledger = env.ledger().sequence() as i128;

        let bond_terms: BondTerms = env
            .storage()
            .persistent()
            .get(&symbol_short!("bt"))
            .expect(ERR_BOND_NOT_FOUND);

        let yield_stream: YieldStream = env
            .storage()
            .persistent()
            .get(&symbol_short!("ys"))
            .expect(ERR_BOND_NOT_FOUND);

        let accrual_start_ledger = yield_stream.accrual_start_ledger as i128;
        let blocks_elapsed = current_ledger - accrual_start_ledger;

        if blocks_elapsed <= 0 {
            return 0;
        }

        let annual_coupon = (bond_terms.face_value * bond_terms.coupon_rate_bps as i128) / 10000;
        let daily_coupon = annual_coupon / 365;
        let coupon_per_block = (daily_coupon * 365) / 75000;
        coupon_per_block * blocks_elapsed
    }

    /// Query escrow status.
    pub fn query_escrow(env: Env, bond_id: Symbol) -> (i128, i128) {
        let escrow_record: EscrowRecord = env
            .storage()
            .persistent()
            .get(&symbol_short!("esc"))
            .expect(ERR_BOND_NOT_FOUND);

        (escrow_record.escrow_balance, escrow_record.required_balance)
    }

    /// Query total minted.
    pub fn query_total_minted(env: Env, bond_id: Symbol) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("tm"))
            .unwrap_or(0)
    }

    /// Query total redeemed.
    pub fn query_total_redeemed(env: Env, bond_id: Symbol) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("tr"))
            .unwrap_or(0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    #[test]
    fn test_bond_issuance() {
        let env = Env::default();
        let issuer = Address::random(&env);
        let payment_token = Address::random(&env);
        let bond_id = symbol_short!("bond1");
        let face_value = 1_000_000i128;
        let maturity_ledger = 1000;
        let coupon_rate_bps = 500;
        let escrow_amount = 1_000_000i128;

        env.ledger().set_sequence_number(100);

        let result = ApexCreditContract::issue_bond(
            env.clone(),
            issuer.clone(),
            bond_id.clone(),
            face_value,
            maturity_ledger,
            coupon_rate_bps,
            payment_token,
            escrow_amount,
        );

        assert_eq!(result, bond_id);
        let terms = ApexCreditContract::query_bond_terms(env.clone(), bond_id.clone());
        assert_eq!(terms.issuer, issuer);
        assert_eq!(terms.face_value, face_value);
        assert_eq!(terms.coupon_rate_bps, coupon_rate_bps);
        assert_eq!(terms.maturity_ledger, maturity_ledger);
    }

    #[test]
    fn test_transfer_bond() {
        let env = Env::default();
        let issuer = Address::random(&env);
        let buyer = Address::random(&env);
        let payment_token = Address::random(&env);
        let bond_id = symbol_short!("bond5");
        let face_value = 1_000_000i128;
        let maturity_ledger = 1000;
        let coupon_rate_bps = 500;
        let escrow_amount = 1_000_000i128;

        env.ledger().set_sequence_number(100);

        ApexCreditContract::issue_bond(
            env.clone(),
            issuer.clone(),
            bond_id.clone(),
            face_value,
            maturity_ledger,
            coupon_rate_bps,
            payment_token,
            escrow_amount,
        );

        ApexCreditContract::transfer(
            env.clone(),
            issuer.clone(),
            buyer.clone(),
            bond_id.clone(),
            face_value / 2,
        );

        let issuer_balance =
            ApexCreditContract::query_balance(env.clone(), bond_id.clone(), issuer);
        let buyer_balance = ApexCreditContract::query_balance(env, bond_id, buyer);

        assert_eq!(issuer_balance, face_value / 2);
        assert_eq!(buyer_balance, face_value / 2);
    }

    #[test]
    fn test_accrued_interest() {
        let env = Env::default();
        let issuer = Address::random(&env);
        let payment_token = Address::random(&env);
        let bond_id = symbol_short!("bond6");
        let face_value = 1_000_000i128;
        let maturity_ledger = 2000;
        let coupon_rate_bps = 10000;
        let escrow_amount = 1_000_000i128;

        env.ledger().set_sequence_number(100);

        ApexCreditContract::issue_bond(
            env.clone(),
            issuer.clone(),
            bond_id.clone(),
            face_value,
            maturity_ledger,
            coupon_rate_bps,
            payment_token,
            escrow_amount,
        );

        env.ledger().set_sequence_number(100 + 75000);

        let accrued = ApexCreditContract::query_accrued_interest(env, bond_id, issuer);
        assert!(accrued > 0);
    }
}
