
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_slow_wallet {
  use aptos_framework::stake;
  use aptos_framework::account;
  use aptos_framework::reconfiguration;
  use ol_framework::slow_wallet;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use ol_framework::gas_coin;
  use aptos_framework::coin;
  use aptos_std::debug::print;
  use std::vector;

  #[test]
  // we are testing that genesis creates the needed struct
  // and a validator creation sets the users account to slow.
  fun slow_wallet_init () {
      let _set = mock::genesis_n_vals(4);
      let list = slow_wallet::get_slow_list();
      print(&list);
      // alice, the validator, is already a slow wallet.
      assert!(vector::length<address>(&list) == 4, 735701);

      // frank was not created above
      let sig = account::create_signer_for_test(@0x1000f);
      let (_sk, pk, pop) = stake::generate_identity();
      stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);

      let list = slow_wallet::get_slow_list();
      assert!(vector::length<address>(&list) == 5, 735701);
    
  }

  #[test(vm=@vm_reserved)]
  fun test_epoch_drip(vm: signer) {
    let set = mock::genesis_n_vals(4);
    let a = vector::borrow(&set, 0);
    let a_sig = account::create_signer_for_test(*a);

    slow_wallet::set_slow(&a_sig);
    assert!(slow_wallet::unlocked_amount(*a) == 0, 735701);

    slow_wallet::slow_wallet_epoch_drip(&vm, 100);
    assert!(slow_wallet::unlocked_amount(*a) == 100, 735702);
  }

  #[test(genesis = @ol_framework, vm = @vm_reserved, alice = @0x123, bob = @0x456)]
  fun test_transfer_happy(genesis: signer, vm: signer, alice: signer) {
    slow_wallet::initialize(&genesis);
    ol_account::create_account(@0x123);


    // let _vm = vm;
    // let set = mock::genesis_n_vals(4);

    // let a_sig = account::create_signer_for_test(*a);

    slow_wallet::set_slow(&alice);
    assert!(slow_wallet::is_slow(@0x123), 7357000);
    assert!(slow_wallet::unlocked_amount(@0x123) == 0, 735701);
    let (burn_cap, mint_cap) = gas_coin::initialize_for_test(&genesis);
    coin::deposit(@0x123, coin::mint(10000, &mint_cap));

    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);
    ol_account::transfer(&alice, @0x456, 99);

    let b_balance = coin::balance<gas_coin::GasCoin>(@0x456);
    assert!(b_balance == 0, 735702);
    slow_wallet::slow_wallet_epoch_drip(&vm, 100);

    assert!(slow_wallet::unlocked_amount(@0x123) == 100, 735703);
    ol_account::transfer(&alice, @0x456, 10);
    let b_balance = coin::balance<gas_coin::GasCoin>(@0x456);
    assert!(b_balance == 10, 735704);
  }

  #[test(genesis = @ol_framework, vm = @vm_reserved, alice = @0x123, bob = @0x456)]
  #[expected_failure(abort_code = 735704, location = Self)]

  fun test_transfer_sad(genesis: signer, vm: signer, alice: signer) {
    slow_wallet::initialize(&genesis);
    ol_account::create_account(@0x123);

    slow_wallet::set_slow(&alice);
    assert!(slow_wallet::is_slow(@0x123), 7357000);
    assert!(slow_wallet::unlocked_amount(@0x123) == 0, 735701);
    let (burn_cap, mint_cap) = gas_coin::initialize_for_test(&genesis);
    coin::deposit(@0x123, coin::mint(10000, &mint_cap));

    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);
    ol_account::transfer(&alice, @0x456, 99);

    let b_balance = coin::balance<gas_coin::GasCoin>(@0x456);
    assert!(b_balance == 0, 735702);
    slow_wallet::slow_wallet_epoch_drip(&vm, 100);

    assert!(slow_wallet::unlocked_amount(@0x123) == 100, 735703);
    ol_account::transfer(&alice, @0x456, 200); // TOO MUCH
    let b_balance = coin::balance<gas_coin::GasCoin>(@0x456);
    assert!(b_balance == 10, 735704);
  }


  #[test(vm = @0x0)]
  // we are testing that genesis creates the needed struct
  // and a validator creation sets the users account to slow.
  fun slow_wallet_reconfigure (vm: signer) {
    let set = mock::genesis_n_vals(4);
    let a = vector::borrow(&set, 0);
    // let a_sig = account::create_signer_for_test(*a);

    // slow_wallet::set_slow(&a_sig);
    assert!(slow_wallet::unlocked_amount(*a) == 0, 735701);
      // let list = slow_wallet::get_slow_list();
    reconfiguration::ol_reconfigure_for_test(&vm)
    
  }


}