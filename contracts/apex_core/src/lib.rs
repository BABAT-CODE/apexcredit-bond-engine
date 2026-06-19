#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec, 
    log, xdr::ToXdr,
};

// ============================================================================
// Error Codes
// ============================================================================

const ERR_UNAUTHORIZED: u32 = 401;
const ERR_INVALID_MATURITY: u32 = 402;
const ERR_INVALID_COUPON_RATE: u32 = 403;
const ERR_INSUFFICIENT_ESCROW: u32 = 404;
const ERR_DUPLICATE_BOND_ID: u32 = 405;
const ERR_NOT_MATURE: u32 = 406;
const ERR_INSUFFICIENT_BALANCE: u32 = 407;
const ERR_BOND_NOT_FOUND: u32 = 408;
const ERR_INVALID_RECIPIENT: u32 = 409;
const ERR_ESCROW_UNDERFUNDED: u32 = 410;

// ============================================================================
// Data Structures
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondTerms {
    pub issuer: Address,
    pub bond_id: Symbol,
    pub face_value: i128,           // In stablecoin smallest unit
    pub maturity_ledger: u32,       // Ledger sequence for maturity
    pub coupon_rate_bps: u32,       // Annual rate in basis points (0-10000)
    pub payment_token: Address,     // Stablecoin contract address
    pub issued_ledger: u32,         // Ledger when bond was issued
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YieldStream {
    pub bond_id: Symbol,
    pub holder: Address,
    pub accrual_start_ledger: u32,  // When yield accrual began for this holder
    pub last_collection_ledger: u32, // Last block yield was streamed
}

// Storage key helpers (inline for optimization)
// Keys follow pattern: "key_name_{bond_id}_{optional_holder}"

// ============================================================================
// Event Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug)]
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
    PrincipalRedeemed {
        bond_id: Symbol,
        holder: Address,
        amount: i128,
        timestamp: u32,
    },
}

// ============================================================================
// Main Contract
// ============================================================================

#[contract]
pub struct ApexCreditEngine;

#[contractimpl]
impl ApexCreditEngine {
    /// Issue a new tokenized corporate bond with fixed terms
    pub fn issue_corporate_bond(
        env: Env,
        issuer: Address,
        bond_id: Symbol,
        face_value: i128,
        maturity_ledger: u32,
        coupon_rate_bps: u32,
        payment_token: Address,
        escrow_amount: i128,
    ) {
        issuer.require_auth();

        // Validation: maturity must be in the future
        if maturity_ledger <= env.ledger().sequence() {
            panic!("ERR_INVALID_MATURITY");
        }

        // Validation: coupon rate must be 0-10000 bps
        if coupon_rate_bps > 10000 {
            panic!("ERR_INVALID_COUPON_RATE");
        }

        // Validation: escrow must be sufficient
        if escrow_amount < face_value {
            panic!("ERR_INSUFFICIENT_ESCROW");
        }

        // Validation: bond ID must not already exist
        let storage = env.storage().persistent();
        if storage.has(&bond_id) {
            panic!("ERR_DUPLICATE_BOND_ID");
        }

        // Transfer escrow from issuer to contract
        let token_client = token::Client::new(&env, &payment_token);
        token_client.transfer(
            &issuer,
            &env.current_contract_address(),
            &escrow_amount,
        );

        // Store BondTerms
        let terms = BondTerms {
            issuer: issuer.clone(),
            bond_id: bond_id.clone(),
            face_value,
            maturity_ledger,
            coupon_rate_bps,
            payment_token,
            issued_ledger: env.ledger().sequence(),
        };
        storage.set(&bond_id, &terms);

        // Store EscrowRecord
        let escrow = EscrowRecord {
            bond_id: bond_id.clone(),
            issuer: issuer.clone(),
            escrow_balance: escrow_amount,
            required_balance: face_value,
            created_ledger: env.ledger().sequence(),
            maturity_ledger,
        };
        storage.set(&format!("escrow_{:?}", bond_id), &escrow);

        // Initialize bondholder balance (issuer owns all bonds initially)
        storage.set(
            &format!("balance_{:?}_{:?}", bond_id, issuer),
            &face_value,
        );

        // Initialize total minted and redeemed
        storage.set(&format!("total_minted_{:?}", bond_id), &face_value);
        storage.set(&format!("total_redeemed_{:?}", bond_id), &0i128);

        // Initialize yield stream for issuer
        let yield_stream = YieldStream {
            bond_id: bond_id.clone(),
            holder: issuer.clone(),
            accrual_start_ledger: env.ledger().sequence(),
            last_collection_ledger: env.ledger().sequence(),
        };
        storage.set(
            &format!("yield_stream_{:?}_{:?}", bond_id, issuer),
            &yield_stream,
        );

        // Emit BondIssued event
        env.events().publish(
            ("apex", "bond_issued"),
            ContractEvent::BondIssued {
                bond_id,
                issuer,
                face_value,
                coupon_rate_bps,
                maturity_ledger,
            },
        );
    }

    /// Redeem bond tokens for principal at maturity
    pub fn redeem_principal(
        env: Env,
        holder: Address,
        bond_id: Symbol,
        amount: i128,
    ) {
        holder.require_auth();

        let storage = env.storage().persistent();

        // Retrieve bond terms
        let terms: BondTerms = match storage.get(&bond_id) {
            Ok(t) => t,
            Err(_) => panic!("ERR_BOND_NOT_FOUND"),
        };

        // Validate maturity is reached
        if env.ledger().sequence() < terms.maturity_ledger {
            panic!("ERR_NOT_MATURE");
        }

        // Check holder balance
        let balance_key = format!("balance_{:?}_{:?}", bond_id, holder);
        let balance: i128 = storage.get(&balance_key).unwrap_or(0);
        if balance < amount {
            panic!("ERR_INSUFFICIENT_BALANCE");
        }

        // Check escrow balance
        let escrow_key = format!("escrow_{:?}", bond_id);
        let mut escrow: EscrowRecord = storage.get(&escrow_key).unwrap();
        if escrow.escrow_balance < amount {
            panic!("ERR_INSUFFICIENT_ESCROW");
        }

        // Burn tokens (reduce bondholder balance)
        storage.set(&balance_key, &(balance - amount));

        // Transfer stablecoin from escrow to holder
        let token_client = token::Client::new(&env, &terms.payment_token);
        token_client.transfer(
            &env.current_contract_address(),
            &holder,
            &amount,
        );

        // Update escrow
        escrow.escrow_balance -= amount;
        storage.set(&escrow_key, &escrow);

        // Update total redeemed
        let redeemed_key = format!("total_redeemed_{:?}", bond_id);
        let total_redeemed: i128 = storage.get(&redeemed_key).unwrap_or(0);
        storage.set(&redeemed_key, &(total_redeemed + amount));

        // Emit PrincipalRedeemed event
        env.events().publish(
            ("apex", "principal_redeemed"),
            ContractEvent::PrincipalRedeemed {
                bond_id,
                holder,
                amount,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    /// Transfer bond tokens between holders (secondary trading)
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        bond_id: Symbol,
        amount: i128,
    ) {
        from.require_auth();

        let storage = env.storage().persistent();

        // Validate bond exists
        let _terms: BondTerms = match storage.get(&bond_id) {
            Ok(t) => t,
            Err(_) => panic!("ERR_BOND_NOT_FOUND"),
        };

        // Check sender balance
        let from_balance_key = format!("balance_{:?}_{:?}", bond_id, from);
        let from_balance: i128 = storage.get(&from_balance_key).unwrap_or(0);
        if from_balance < amount {
            panic!("ERR_INSUFFICIENT_BALANCE");
        }

        // Update balances
        storage.set(&from_balance_key, &(from_balance - amount));

        let to_balance_key = format!("balance_{:?}_{:?}", bond_id, to);
        let to_balance: i128 = storage.get(&to_balance_key).unwrap_or(0);
        storage.set(&to_balance_key, &(to_balance + amount));

        // Update yield streams
        let from_yield_key = format!("yield_stream_{:?}_{:?}", bond_id, from);
        let to_yield_key = format!("yield_stream_{:?}_{:?}", bond_id, to);

        // Stop yield stream for sender (if fully transferred)
        if from_balance == amount {
            storage.remove(&from_yield_key);
        }

        // Create new yield stream for recipient
        let yield_stream = YieldStream {
            bond_id: bond_id.clone(),
            holder: to.clone(),
            accrual_start_ledger: env.ledger().sequence(),
            last_collection_ledger: env.ledger().sequence(),
        };
        storage.set(&to_yield_key, &yield_stream);

        // Emit Transfer event
        env.events().publish(
            ("apex", "transfer"),
            ContractEvent::Transfer {
                from,
                to,
                bond_id,
                amount,
            },
        );
    }

    /// Query bond terms
    pub fn query_bond_terms(env: Env, bond_id: Symbol) -> BondTerms {
        let storage = env.storage().persistent();
        match storage.get(&bond_id) {
            Ok(terms) => terms,
            Err(_) => panic!("ERR_BOND_NOT_FOUND"),
        }
    }

    /// Query bondholder balance
    pub fn query_bondholder_balance(
        env: Env,
        bond_id: Symbol,
        holder: Address,
    ) -> i128 {
        let storage = env.storage().persistent();
        let key = format!("balance_{:?}_{:?}", bond_id, holder);
        storage.get(&key).unwrap_or(0)
    }

    /// Query accrued interest for a bondholder
    pub fn query_accrued_interest(
        env: Env,
        bond_id: Symbol,
        holder: Address,
    ) -> i128 {
        let storage = env.storage().persistent();

        // Retrieve bond terms
        let terms: BondTerms = match storage.get(&bond_id) {
            Ok(t) => t,
            Err(_) => return 0,
        };

        // Retrieve yield stream
        let yield_key = format!("yield_stream_{:?}_{:?}", bond_id, holder);
        let yield_stream: YieldStream = match storage.get(&yield_key) {
            Ok(y) => y,
            Err(_) => return 0,
        };

        // Calculate accrued interest
        // Formula: face_value × (coupon_rate_bps / 10000) × blocks_elapsed / (365 × 75000)
        let current_ledger = env.ledger().sequence();
        let blocks_elapsed = (current_ledger - yield_stream.accrual_start_ledger) as i128;

        let annual_coupon = (terms.face_value * terms.coupon_rate_bps as i128) / 10000;
        let daily_coupon = annual_coupon / 365;
        let blocks_per_year = 365i128 * 75000i128; // ~75000 blocks per year on Stellar
        let accrued = (daily_coupon * blocks_elapsed) / blocks_per_year;

        accrued
    }

    /// Query escrow status
    pub fn query_escrow(env: Env, bond_id: Symbol) -> (i128, i128, Symbol) {
        let storage = env.storage().persistent();
        let key = format!("escrow_{:?}", bond_id);

        let escrow: EscrowRecord = match storage.get(&key) {
            Ok(e) => e,
            Err(_) => panic!("ERR_BOND_NOT_FOUND"),
        };

        let status = if env.ledger().sequence() > escrow.maturity_ledger {
            Symbol::new(&env, "MATURED")
        } else if escrow.escrow_balance < escrow.required_balance {
            Symbol::new(&env, "UNDERFUNDED")
        } else {
            Symbol::new(&env, "ACTIVE")
        };

        (escrow.escrow_balance, escrow.required_balance, status)
    }

    /// Query total bonds minted
    pub fn query_total_minted(env: Env, bond_id: Symbol) -> i128 {
        let storage = env.storage().persistent();
        let key = format!("total_minted_{:?}", bond_id);
        storage.get(&key).unwrap_or(0)
    }

    /// Query total bonds redeemed
    pub fn query_total_redeemed(env: Env, bond_id: Symbol) -> i128 {
        let storage = env.storage().persistent();
        let key = format!("total_redeemed_{:?}", bond_id);
        storage.get(&key).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn test_issue_bond_success() {
        let env = Env::default();
        let issuer = Address::random(&env);
        let bond_id = Symbol::new(&env, "BOND_001");
        let payment_token = Address::random(&env);

        let contract = ApexCreditEngine;
        // This test would require more setup with token contracts
        // Simplified for demonstration
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_MATURITY")]
    fn test_issue_bond_past_maturity() {
        let env = Env::default();
        env.ledger().set_sequence(1000);

        let issuer = Address::random(&env);
        let bond_id = Symbol::new(&env, "BOND_001");
        let payment_token = Address::random(&env);

        // Would fail with ERR_INVALID_MATURITY since maturity < current ledger
    }
}
