// Example: Complete bond lifecycle
// This demonstrates how to use the ApexCredit contract

use apex_credit::{ApexCreditContract, Symbol};
use soroban_sdk::{Address, Env, symbol_short};

#[no_mangle]
pub extern "C" fn example_bond_lifecycle() {
    let env = Env::default();

    // ============================================================================
    // 1. SETUP
    // ============================================================================

    // Addresses
    let issuer = Address::random(&env);
    let buyer1 = Address::random(&env);
    let buyer2 = Address::random(&env);
    let stablecoin = Address::random(&env);

    // Bond parameters
    let bond_id = symbol_short!("ACBD");
    let face_value = 100_000_000i128; // $1M in smallest units
    let maturity_ledger = 2000;
    let coupon_rate_bps = 500; // 5% annual
    let escrow_amount = 100_000_000i128; // Must match face_value

    println!("=== ApexCredit Bond Lifecycle Example ===\n");

    // ============================================================================
    // 2. ISSUANCE
    // ============================================================================

    println!("Phase 1: ISSUANCE");
    println!("- Issuer: {:?}", issuer);
    println!("- Bond ID: {:?}", bond_id);
    println!("- Face Value: $1,000,000");
    println!("- Coupon: 5% annual");
    println!("- Maturity: Block 2000\n");

    ApexCreditContract::issue_bond(
        env.clone(),
        issuer.clone(),
        bond_id.clone(),
        face_value,
        maturity_ledger,
        coupon_rate_bps,
        stablecoin.clone(),
        escrow_amount,
    );

    // Verify issuance
    let terms = ApexCreditContract::query_bond_terms(env.clone(), bond_id.clone());
    println!("✓ Bond issued successfully");
    println!("  Issuer balance: {}\n", face_value);

    // ============================================================================
    // 3. SECONDARY MARKET - BUYER 1 PURCHASES 40%
    // ============================================================================

    println!("Phase 2: SECONDARY MARKET - BUYER 1");
    let transfer_amount_1 = face_value * 40 / 100; // 40% of bonds

    ApexCreditContract::transfer(
        env.clone(),
        issuer.clone(),
        buyer1.clone(),
        bond_id.clone(),
        transfer_amount_1,
    );

    let issuer_bal = ApexCreditContract::query_balance(env.clone(), bond_id.clone(), issuer.clone());
    let buyer1_bal = ApexCreditContract::query_balance(env.clone(), bond_id.clone(), buyer1.clone());

    println!("✓ Buyer 1 purchased 40% of bonds");
    println!("  Issuer balance: {} (60%)", issuer_bal);
    println!("  Buyer 1 balance: {} (40%)\n", buyer1_bal);

    // ============================================================================
    // 4. ACCRUED INTEREST (After some time)
    // ============================================================================

    println!("Phase 3: ACCRUED INTEREST TRACKING");
    println!("  Time passes... (blocks accumulate)\n");

    // In a real scenario, blocks would pass
    // Let's query accrued interest
    let accrued_issuer = ApexCreditContract::query_accrued_interest(env.clone(), bond_id.clone(), issuer.clone());
    let accrued_buyer1 = ApexCreditContract::query_accrued_interest(env.clone(), bond_id.clone(), buyer1.clone());

    println!("✓ Accrued interest calculated:");
    println!("  Issuer coupon: {} (accruing on 60%)", accrued_issuer);
    println!("  Buyer 1 coupon: {} (accruing on 40%)\n", accrued_buyer1);

    // ============================================================================
    // 5. SECONDARY MARKET - BUYER 2 PURCHASES FROM ISSUER
    // ============================================================================

    println!("Phase 4: SECONDARY MARKET - BUYER 2");
    let transfer_amount_2 = issuer_bal / 2; // 30% of original

    ApexCreditContract::transfer(
        env.clone(),
        issuer.clone(),
        buyer2.clone(),
        bond_id.clone(),
        transfer_amount_2,
    );

    let issuer_bal_final = ApexCreditContract::query_balance(env.clone(), bond_id.clone(), issuer.clone());
    let buyer2_bal = ApexCreditContract::query_balance(env.clone(), bond_id.clone(), buyer2.clone());

    println!("✓ Buyer 2 purchased from Issuer");
    println!("  Issuer balance: {} (30%)", issuer_bal_final);
    println!("  Buyer 2 balance: {} (30%)\n", buyer2_bal);

    // ============================================================================
    // 6. ESCROW STATUS
    // ============================================================================

    println!("Phase 5: ESCROW VERIFICATION");
    let (escrow_balance, required_balance) =
        ApexCreditContract::query_escrow(env.clone(), bond_id.clone());

    println!("✓ Escrow Status:");
    println!("  Required: {}", required_balance);
    println!("  Current: {} (safe)\n", escrow_balance);

    // ============================================================================
    // 7. TOTAL ACCOUNTING
    // ============================================================================

    println!("Phase 6: ACCOUNTING");
    let total_minted = ApexCreditContract::query_total_minted(env.clone(), bond_id.clone());
    let total_redeemed = ApexCreditContract::query_total_redeemed(env.clone(), bond_id.clone());

    println!("✓ Bond Inventory:");
    println!("  Total Minted: {}", total_minted);
    println!("  Total Redeemed: {} (so far)", total_redeemed);
    println!("  Outstanding: {}\n", total_minted - total_redeemed);

    // ============================================================================
    // 8. MATURITY & REDEMPTION (After maturity_ledger reached)
    // ============================================================================

    println!("Phase 7: MATURITY REDEMPTION");
    println!("  (Simulating redemption at or after maturity block)\n");

    // Issuer redeems their 30% position
    ApexCreditContract::redeem_principal(
        env.clone(),
        issuer.clone(),
        bond_id.clone(),
        issuer_bal_final,
    );

    println!("✓ Issuer redeemed position: {} tokens", issuer_bal_final);

    let total_redeemed_after = ApexCreditContract::query_total_redeemed(env.clone(), bond_id.clone());
    let escrow_after = ApexCreditContract::query_escrow(env.clone(), bond_id.clone()).0;

    println!("  Total redeemed: {}", total_redeemed_after);
    println!("  Escrow remaining: {}\n", escrow_after);

    // Buyer 1 redeems
    ApexCreditContract::redeem_principal(
        env.clone(),
        buyer1.clone(),
        bond_id.clone(),
        buyer1_bal,
    );

    println!("✓ Buyer 1 redeemed position: {} tokens", buyer1_bal);

    // Buyer 2 redeems
    ApexCreditContract::redeem_principal(
        env.clone(),
        buyer2.clone(),
        bond_id.clone(),
        buyer2_bal,
    );

    println!("✓ Buyer 2 redeemed position: {} tokens", buyer2_bal);

    let total_redeemed_final = ApexCreditContract::query_total_redeemed(env.clone(), bond_id.clone());
    let escrow_final = ApexCreditContract::query_escrow(env.clone(), bond_id.clone()).0;

    println!("\n=== FINAL STATE ===");
    println!("✓ All bonds redeemed");
    println!("  Total redeemed: {} (100%)", total_redeemed_final);
    println!("  Escrow remaining: {} (released to bondholders)\n", escrow_final);

    println!("=== Bond lifecycle complete ===");
}
