

use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use zapatos_forge::Swarm;
use zapatos_types::transaction::Script;
use std::path::PathBuf;
// use zapatos_crypto::traits::ValidCryptoMaterialStringExt;

use libra_cached_packages::aptos_stdlib::{aptos_governance_ol_create_proposal_v2, aptos_governance_ol_vote, aptos_governance_can_resolve};
use zapatos_sdk::types::LocalAccount;

use crate::helpers::mint_libra;

#[tokio::test]
async fn can_submit_proposal() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();

    let mut swarm = new_local_swarm_with_release(4, release).await;

    let v = swarm.validators_mut().next().unwrap();
    let pri_key = v.account_private_key().as_ref().unwrap();
    
    let mut account = LocalAccount::new(v.peer_id(), pri_key.private_key(), 0);
    
    let proposal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script.mv");
    dbg!(&proposal_path);
    assert!(&proposal_path.exists());

    let proposal_hash_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script_sha3");
    assert!(&proposal_hash_path.exists());

    // note: this proposal hash was generated by the
    // framework::main::upgrade path and added here to test fixtures.
    let proposal_hash = std::fs::read(proposal_hash_path).unwrap();
    dbg!(&proposal_hash);
    
    let payload = aptos_governance_ol_create_proposal_v2(
        // v.peer_id(),
        proposal_hash,
        "metadata url".to_string().as_bytes().to_vec(),
        "metadata struct".to_string().as_bytes().to_vec(),
        true,
    );

    let public_info: zapatos_forge::AptosPublicInfo = swarm.aptos_public_info();

    // make sure we have gas

    let txn = account.sign_with_transaction_builder(
        public_info.transaction_factory()
            .payload(payload),
    );

    let res = public_info.client().submit_and_wait(&txn).await.unwrap();
    dbg!(&res);

    // check the network still runs
    // check_create_mint_transfer(&mut env).await;
}


#[tokio::test]
async fn can_vote() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();

    let mut swarm = new_local_swarm_with_release(4, release).await;

    // Create accounts. Needs to happen up top because of borrowing swarm
    let alice = swarm.validators().nth(0).unwrap();
    let pri_key = alice.account_private_key().as_ref().unwrap();
    let mut alice_account = LocalAccount::new(alice.peer_id(), pri_key.private_key(), 0);

    let bob = swarm.validators().nth(1).unwrap();
    let pri_key = bob.account_private_key().as_ref().unwrap();
    let mut bob_account = LocalAccount::new(bob.peer_id(), pri_key.private_key(), 0);

    let carol = swarm.validators().nth(2).unwrap();
    let pri_key = carol.account_private_key().as_ref().unwrap();
    let mut carol_account = LocalAccount::new(carol.peer_id(), pri_key.private_key(), 0);

    let dave = swarm.validators().nth(3).unwrap();
    let pri_key = dave.account_private_key().as_ref().unwrap();
    let mut dave_account = LocalAccount::new(dave.peer_id(), pri_key.private_key(), 0);
    //////// end create accounts
    
    let proposal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script.mv");
    // dbg!(&proposal_path);
    assert!(&proposal_path.exists());

    let proposal_hash_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script_sha3");
    assert!(&proposal_hash_path.exists());

    // note: this proposal hash was generated by the
    // framework::main::upgrade path and added here to test fixtures.
    let proposal_hash = std::fs::read(proposal_hash_path).unwrap();
    // dbg!(&proposal_hash);
    
    let payload = aptos_governance_ol_create_proposal_v2(
        // alice.peer_id(),
        proposal_hash,
        "metadata url".to_string().as_bytes().to_vec(),
        "metadata struct".to_string().as_bytes().to_vec(),
        true,
    );

    let public_info: zapatos_forge::AptosPublicInfo = swarm.aptos_public_info();

    let txn = alice_account.sign_with_transaction_builder(
        public_info.transaction_factory()
            .payload(payload),
    );
    dbg!("proposal txn");

    public_info.client().submit_and_wait(&txn).await.unwrap();
    // dbg!(&res);
    let proposal_id = 0;
    let vote_payload = aptos_governance_ol_vote(
        proposal_id,
        true, // should_pass
    );
    
    let built_tx = public_info.transaction_factory()
            .payload(vote_payload.clone());
    // Alice submits. Note: the sequence number incremented.
    let txn = alice_account.sign_with_transaction_builder(built_tx);

    dbg!("alice votes");
    // needs gas
    // mint_libra(&mut public_info, alice_account.address(), 10_000_000_000).await.unwrap();
    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");
  
    // TODO: make this a for_each, and solve the error[E0507]: cannot move out of `public_info`, a captured variable in an `FnMut` closure
    // BOB
    dbg!("bob votes");
    let built_tx = public_info
      .transaction_factory()
      .payload(vote_payload.clone());
    let txn = bob_account.sign_with_transaction_builder(built_tx);

    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");

    // CAROL
    dbg!("carol votes");
    let built_tx = public_info
      .transaction_factory()
      .payload(vote_payload.clone());
    let txn = carol_account.sign_with_transaction_builder(built_tx);

    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");


    // DAVE
    dbg!("dave votes");
    let built_tx = public_info
      .transaction_factory()
      .payload(vote_payload);
    let txn = dave_account.sign_with_transaction_builder(built_tx);

    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");


    // check the state of voting

    let check_vote_payload = aptos_governance_can_resolve(proposal_id);

    let built_tx = public_info
      .transaction_factory()
      .payload(check_vote_payload);
    let txn = alice_account.sign_with_transaction_builder(built_tx);
    public_info.client().submit_and_wait(&txn).await.unwrap();

}


#[tokio::test]
async fn can_read_tx() {
    
    let proposal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script.mv");
    // dbg!(&proposal_path);
    assert!(&proposal_path.exists());

    let proposal_bytes = std::fs::read(proposal_path).unwrap();

    let _proposal_tx = Script::new(proposal_bytes, vec![], vec![]);
    
}

#[tokio::test]
async fn can_upgrade() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();

    let mut swarm = new_local_swarm_with_release(4, release).await;

    // Create accounts. Needs to happen up top because of borrowing swarm
    let alice = swarm.validators().nth(0).unwrap();
    let pri_key = alice.account_private_key().as_ref().unwrap();
    let mut alice_account = LocalAccount::new(alice.peer_id(), pri_key.private_key(), 0);

    let bob = swarm.validators().nth(1).unwrap();
    let pri_key = bob.account_private_key().as_ref().unwrap();
    let mut bob_account = LocalAccount::new(bob.peer_id(), pri_key.private_key(), 0);

    let carol = swarm.validators().nth(2).unwrap();
    let pri_key = carol.account_private_key().as_ref().unwrap();
    let mut carol_account = LocalAccount::new(carol.peer_id(), pri_key.private_key(), 0);

    let dave = swarm.validators().nth(3).unwrap();
    let pri_key = dave.account_private_key().as_ref().unwrap();
    let mut dave_account = LocalAccount::new(dave.peer_id(), pri_key.private_key(), 0);
    //////// end create accounts
    
    let proposal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script.mv");
    // dbg!(&proposal_path);
    assert!(&proposal_path.exists());

    let proposal_hash_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script_sha3");
    assert!(&proposal_hash_path.exists());

    // note: this proposal hash was generated by the
    // framework::main::upgrade path and added here to test fixtures.
    let proposal_hash = std::fs::read(proposal_hash_path).unwrap();
    // dbg!(&proposal_hash);
    
    let payload = aptos_governance_ol_create_proposal_v2(
        // alice.peer_id(),
        proposal_hash,
        "metadata url".to_string().as_bytes().to_vec(),
        "metadata struct".to_string().as_bytes().to_vec(),
        true,
    );

    let mut public_info: zapatos_forge::AptosPublicInfo = swarm.aptos_public_info();

    let txn = alice_account.sign_with_transaction_builder(
        public_info.transaction_factory()
            .payload(payload),
    );
    dbg!("proposal txn");

    public_info.client().submit_and_wait(&txn).await.unwrap();
    // dbg!(&res);
    let proposal_id = 0;
    let vote_payload = aptos_governance_ol_vote(
        proposal_id,
        true, // should_pass
    );
    
    let built_tx = public_info.transaction_factory()
            .payload(vote_payload.clone());
    // Alice submits. Note: the sequence number incremented.
    let txn = alice_account.sign_with_transaction_builder(built_tx);

    dbg!("alice votes");
    // needs gas
    // mint_libra(&mut public_info, alice_account.address(), 10_000_000_000).await.unwrap();
    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");
  
    // TODO: make this a for_each, and solve the error[E0507]: cannot move out of `public_info`, a captured variable in an `FnMut` closure
    // BOB
    dbg!("bob votes");
    let built_tx = public_info
      .transaction_factory()
      .payload(vote_payload.clone());
    let txn = bob_account.sign_with_transaction_builder(built_tx);

    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");

    // CAROL
    dbg!("carol votes");
    let built_tx = public_info
      .transaction_factory()
      .payload(vote_payload.clone());
    let txn = carol_account.sign_with_transaction_builder(built_tx);

    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");


    // DAVE
    dbg!("dave votes");
    let built_tx = public_info
      .transaction_factory()
      .payload(vote_payload);
    let txn = dave_account.sign_with_transaction_builder(built_tx);

    public_info.client().submit_and_wait(&txn).await.expect("could not send tx");


    // check the state of voting

    let check_vote_payload = aptos_governance_can_resolve(proposal_id);

    let built_tx = public_info
      .transaction_factory()
      .payload(check_vote_payload);
    let txn = alice_account.sign_with_transaction_builder(built_tx);
    public_info.client().submit_and_wait(&txn).await.unwrap();


    // send the proposal.
    let proposal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("tests").join("fixtures").join("example_proposal_script").join("script.mv");
    // dbg!(&proposal_path);
    assert!(&proposal_path.exists());

    let proposal_bytes = std::fs::read(proposal_path).unwrap();

    let proposal_script = Script::new(proposal_bytes, vec![], vec![]);
    let built_tx = public_info
      .transaction_factory()
      .script(proposal_script);

      // .payload(vote_payload);
    let txn = dave_account.sign_with_transaction_builder(built_tx);

    public_info.client().submit_and_wait(&txn).await.unwrap();


    // make sure the network is functioning after the upgrade
    mint_libra(&mut public_info, dave_account.address(), 10_000_000_000).await.unwrap();

    // TODO: test with a transaction that only exists in the new version of the code.


}