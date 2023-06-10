
module ol_framework::gas_coin {
    use std::string;
    use std::error;
    use std::signer;
    use std::vector;
    use std::option::{Self, Option};

    use aptos_framework::coin::{Self, MintCapability};
    use aptos_framework::system_addresses;

    #[test_only]
    use aptos_framework::coin::BurnCapability;

    friend aptos_framework::genesis;

    /// Account does not have mint capability
    const ENO_CAPABILITIES: u64 = 1;
    /// Mint capability has already been delegated to this specified address
    const EALREADY_DELEGATED: u64 = 2;
    /// Cannot find delegation of mint capability to this account
    const EDELEGATION_NOT_FOUND: u64 = 3;

    struct GasCoin has key {}

    struct MintCapStore has key {
        mint_cap: MintCapability<GasCoin>,
    }

    /// Delegation token created by delegator and can be claimed by the delegatee as MintCapability.
    struct DelegatedMintCapability has store {
        to: address
    }

    /// The container stores the current pending delegations.
    struct Delegations has key {
        inner: vector<DelegatedMintCapability>,
    }

    /// Can only called during genesis to initialize the Aptos coin.
    public(friend) fun initialize(aptos_framework: &signer) {
        system_addresses::assert_aptos_framework(aptos_framework);

        let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<GasCoin>(
            aptos_framework,
            string::utf8(b"Gas Coin"),
            string::utf8(b"GAS"),
            8, /* decimals */
            true, /* monitor_supply */
        );

        // Aptos framework needs mint cap to mint coins to initial validators. This will be revoked once the validators
        // have been initialized.
        move_to(aptos_framework, MintCapStore { mint_cap });

        coin::destroy_freeze_cap(freeze_cap);
        coin::destroy_burn_cap(burn_cap);
        // (burn_cap, mint_cap)
    }

    public fun has_mint_capability(account: &signer): bool {
        exists<MintCapStore>(signer::address_of(account))
    }

    /// Only called during genesis to destroy the aptos framework account's mint capability once all initial validators
    /// and accounts have been initialized during genesis.
    public(friend) fun destroy_mint_cap(aptos_framework: &signer) acquires MintCapStore {
        system_addresses::assert_aptos_framework(aptos_framework);
        let MintCapStore { mint_cap } = move_from<MintCapStore>(@aptos_framework);
        coin::destroy_mint_cap(mint_cap);
    }

    public(friend) fun restore_mint_cap(aptos_framework: &signer, mint_cap: MintCapability<GasCoin>) {
        system_addresses::assert_aptos_framework(aptos_framework);
        move_to(aptos_framework, MintCapStore { mint_cap });
    }

    /// FOR TESTS ONLY
    /// The `core addresses` sudo account is used to execute system transactions for testing
    /// Can only be called during genesis for tests to grant mint capability to aptos framework and core resources
    /// accounts.
    public(friend) fun configure_accounts_for_test(
        aptos_framework: &signer,
        core_resources: &signer,
        // mint_cap: MintCapability<GasCoin>,
    ) {
        system_addresses::assert_aptos_framework(aptos_framework);

        let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<GasCoin>(
            aptos_framework,
            string::utf8(b"Gas Coin"),
            string::utf8(b"GAS"),
            8, /* decimals */
            true, /* monitor_supply */
        );

        // Mint the core resource account GasCoin for gas so it can execute system transactions.
        coin::register<GasCoin>(core_resources);
        // mint(aptos_framework, signer::address_of(core_resources), 18446744073709551615);
        let coins = coin::mint<GasCoin>(
            18446744073709551615,
            &mint_cap,
        );
        coin::deposit<GasCoin>(signer::address_of(core_resources), coins);

        move_to(core_resources, MintCapStore { mint_cap });
        move_to(core_resources, Delegations { inner: vector::empty() });

        coin::destroy_freeze_cap(freeze_cap);
        coin::destroy_burn_cap(burn_cap);
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Create new coins and deposit them into dst_addr's account.
    public entry fun mint(
        account: &signer,
        dst_addr: address,
        amount: u64,
    ) acquires MintCapStore {
        let account_addr = signer::address_of(account);

        assert!(
            exists<MintCapStore>(account_addr),
            error::not_found(ENO_CAPABILITIES),
        );

        let mint_cap = &borrow_global<MintCapStore>(account_addr).mint_cap;
        let coins_minted = coin::mint<GasCoin>(amount, mint_cap);
        coin::deposit<GasCoin>(dst_addr, coins_minted);
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Create delegated token for the address so the account could claim MintCapability later.
    public entry fun delegate_mint_capability(account: signer, to: address) acquires Delegations {
        system_addresses::assert_core_resource(&account);
        let delegations = &mut borrow_global_mut<Delegations>(@core_resources).inner;
        let i = 0;
        while (i < vector::length(delegations)) {
            let element = vector::borrow(delegations, i);
            assert!(element.to != to, error::invalid_argument(EALREADY_DELEGATED));
            i = i + 1;
        };
        vector::push_back(delegations, DelegatedMintCapability { to });
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Claim the delegated mint capability and destroy the delegated token.
    public entry fun claim_mint_capability(account: &signer) acquires Delegations, MintCapStore {
        let maybe_index = find_delegation(signer::address_of(account));
        assert!(option::is_some(&maybe_index), EDELEGATION_NOT_FOUND);
        let idx = *option::borrow(&maybe_index);
        let delegations = &mut borrow_global_mut<Delegations>(@core_resources).inner;
        let DelegatedMintCapability { to: _ } = vector::swap_remove(delegations, idx);

        // Make a copy of mint cap and give it to the specified account.
        let mint_cap = borrow_global<MintCapStore>(@core_resources).mint_cap;
        move_to(account, MintCapStore { mint_cap });
    }

    fun find_delegation(addr: address): Option<u64> acquires Delegations {
        let delegations = &borrow_global<Delegations>(@core_resources).inner;
        let i = 0;
        let len = vector::length(delegations);
        let index = option::none();
        while (i < len) {
            let element = vector::borrow(delegations, i);
            if (element.to == addr) {
                index = option::some(i);
                break
            };
            i = i + 1;
        };
        index
    }

    #[test_only]
    use aptos_framework::aggregator_factory;

    #[test_only]
    public fun initialize_for_test(aptos_framework: &signer): (BurnCapability<GasCoin>, MintCapability<GasCoin>) {
        aggregator_factory::initialize_aggregator_factory_for_test(aptos_framework);
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<GasCoin>(
            aptos_framework,
            string::utf8(b"Gas Coin"),
            string::utf8(b"GAS"),
            8, /* decimals */
            true, /* monitor_supply */
        );
        coin::destroy_freeze_cap(freeze_cap);
        (burn_cap, mint_cap)
    }

    // This is particularly useful if the aggregator_factory is already initialized via another call path.
    #[test_only]
    public fun initialize_for_test_without_aggregator_factory(aptos_framework: &signer): (BurnCapability<GasCoin>, MintCapability<GasCoin>) {
                let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<GasCoin>(
            aptos_framework,
            string::utf8(b"Gas Coin"),
            string::utf8(b"GAS"),
            8, /* decimals */
            true, /* monitor_supply */
        );
        coin::destroy_freeze_cap(freeze_cap);
        (burn_cap, mint_cap)
    }
}
