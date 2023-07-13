
  /// DonorDirected wallet governance. See documentation at DonorDirected.move


  /// For each DonorDirected account there are Donors.
  /// We establish who is a Donor through the Receipts module.
  /// The DonorDirected account also has a tracker for the Cumulative amount of funds that have been sent to this account.
  /// We will use the lifetime cumulative amounts sent as the total amount of votes that can be cast (voter enrollment).

  /// The voting on a veto of a transaction or an outright liquidation of the account is done by the Donors.
  /// The voting mechanism is a TurnoutTally. Such votes ajust the threshold for passing a vote based on the actual turnout. I.e. The fewer people that vote, the higher the threshold to reach consensus. But a vote is not scuttled if the turnout is low. See more details in the TurnoutTally.move module.
module ol_framework::donor_directed_governance {
    friend ol_framework::donor_directed;

    use std::error;
    use std::signer;
    use std::guid;
    use std::option::{Self, Option};
    use ol_framework::receipts;
    use ol_framework::turnout_tally::{Self, TurnoutTally};
    use ol_framework::ballot::{Self, BallotTracker};
    // use ol_framework::DiemAccount;
    use aptos_framework::account;
    use aptos_framework::reconfiguration;
    use std::vector;
    // use ol_framework::Debug::print;

    /// Is not a donor to this account
    const ENOT_A_DONOR: u64 = 220000;
    /// No ballot found under that GUID
    const ENO_BALLOT_FOUND: u64 = 220001;

    /// Data struct to store all the governance Ballots for vetos
    /// allows for a generic type of Governance action, using the Participation Vote Poll type to keep track of ballots
    struct Governance<T> has key {
      tracker: BallotTracker<T>,
    }

    /// this is a GovAction type for veto
    struct Veto has drop, store {
      guid: guid::ID,
    }

    /// this is a GovAction type for liquidation
    struct Liquidate has drop, store {}



    public fun init_donor_governance(directed_account: &signer) {

      // let t = turnout_tally::new_tally_struct();
      let veto = Governance<TurnoutTally<Veto>> {
          tracker: ballot::new_tracker()
      };

      move_to(directed_account, veto);

      let liquidate = Governance<TurnoutTally<Liquidate>> {
          tracker: ballot::new_tracker()
      };

      move_to(directed_account, liquidate);
    }

    /// For a DonorDirected account get the total number of votes enrolled from reading the Cumulative tracker.
    fun get_enrollment(_directed_account: address): u64 {
      // TODO!
      0
      // DiemAccount::get_cumulative_deposits(directed_account)
    }

    /// public function to check that a user account is a Donor for a DonorDirected account.

    public fun check_is_donor(directed_account: address, user: address): bool {
      get_user_donations(directed_account, user) > 0
    }

    public fun assert_authorized(sig: &signer, directed_account: address) {
      let user = signer::address_of(sig);
      assert!(check_is_donor(directed_account, user), error::permission_denied(ENOT_A_DONOR));
    }

    public fun is_authorized(user: address, directed_account: address):bool {
      check_is_donor(directed_account, user)
    }

    /// For an individual donor, get the amount of votes that they can cast, based on their cumulative donations to the DonorDirected account.

    fun get_user_donations(directed_account: address, user: address): u64 {
      let (_, _, cumulative_donations) = receipts::read_receipt(user, directed_account);

      cumulative_donations
    }


    //////// VETO FUNCTIONS ////////


    /// private function to vote on a ballot based on a Donor's voting power.
    fun vote_veto(user: &signer, ballot: &mut TurnoutTally<Veto>, uid: &guid::ID, multisig_address: address): Option<bool> {
      let user_votes = get_user_donations(multisig_address, signer::address_of(user));

      let veto_tx = true; // True means  approve the ballot, meaning: "veto transaction". Rejecting the ballot would mean "approve transaction".

      turnout_tally::vote<Veto>(user, ballot, uid, veto_tx, user_votes)
    }

  /// Liquidation tally only. The handler for liquidation exists in DonorDirected, where a tx script will call it.
  public(friend) fun vote_liquidation(donor: &signer, multisig_address: address): Option<bool> acquires Governance{
    assert_authorized(donor, multisig_address);
    let state = borrow_global_mut<Governance<TurnoutTally<Liquidate>>>(multisig_address);

    // for liquidation there is only ever one proposal, which never expires
    // so always taket the first one from pending.
    let pending_list = ballot::get_list_ballots_by_enum_mut(&mut state.tracker, ballot::get_pending_enum());
    // print(pending_list);

    if (vector::is_empty(pending_list)) {
      return option::none<bool>()
    };

    let ballot = vector::borrow_mut(pending_list, 0);
    let ballot_guid = ballot::get_ballot_id(ballot);
    let tally_state = ballot::get_type_struct_mut(ballot);
    let user_weight = get_user_donations(multisig_address, signer::address_of(donor));

    turnout_tally::vote(donor, tally_state, &ballot_guid, true, user_weight)
  }



    //////// API ////////

        /// Public script transaction to propose a veto, or vote on it if it already exists.

    /// should only be called by the DonorDirected.move so that the handlers can be called on "pass" conditions.

    public(friend) fun veto_by_id(user: &signer, proposal_guid: &guid::ID): Option<bool> acquires Governance {
      let directed_account = guid::id_creator_address(proposal_guid);
      assert_authorized(user, directed_account);

      let state = borrow_global_mut<Governance<TurnoutTally<Veto>>>(directed_account);

      let ballot = ballot::get_ballot_by_id_mut(&mut state.tracker, proposal_guid);
      let tally_state = ballot::get_type_struct_mut(ballot);

      vote_veto(user, tally_state, proposal_guid, directed_account)
    }

    public(friend) fun sync_ballot_and_tx_expiration(user: &signer, proposal_guid: &guid::ID, epoch_deadline: u64) acquires Governance {
      let directed_account = guid::id_creator_address(proposal_guid);
      assert_authorized(user, directed_account);

      let state = borrow_global_mut<Governance<TurnoutTally<Veto>>>(directed_account);

      let ballot = ballot::get_ballot_by_id_mut(&mut state.tracker, proposal_guid);
      let tally_state = ballot::get_type_struct_mut(ballot);

      turnout_tally::extend_deadline(tally_state, epoch_deadline);

    }

    /// only DonorDirected can call this. The veto and liquidate handlers need
    /// to be located there. So users should not call functions here.
    public(friend) fun propose_veto(
      cap: &account::GUIDCapability,
      guid: &guid::ID, // Id of initiated transaction.
      epochs_duration: u64
    ) acquires Governance {
      let data = Veto { guid: *guid };
      propose_gov<Veto>(cap, data, epochs_duration);
    }

    public(friend) fun propose_liquidate(
      cap: &account::GUIDCapability,
      epochs_duration: u64
    ) acquires Governance {
      let data = Liquidate { };
      propose_gov<Liquidate>(cap, data, epochs_duration);
    }

    /// a private function to propose a ballot for a veto. This is called by a verified donor.

    fun propose_gov<GovAction: drop + store>(cap: &account::GUIDCapability, data: GovAction, epochs_duration: u64) acquires Governance {
      let directed_account = account::get_guid_capability_address(cap);
      let gov_state = borrow_global_mut<Governance<TurnoutTally<GovAction>>>(directed_account);

      if (!is_unique_proposal(&gov_state.tracker, &data)) return;

      // what's the maximum universe of valid votes.
      let max_votes_enrollment = get_enrollment(directed_account);
      if (epochs_duration < 7) {
        epochs_duration = 7;
      };

      let deadline = reconfiguration::get_current_epoch() + epochs_duration; // 7 epochs is about 1 week
      let max_extensions = 0; // infinite

      let t = turnout_tally::new_tally_struct(
        data,
        max_votes_enrollment,
        deadline,
        max_extensions
      );

      let guid = account::create_guid_with_capability(cap);

      ballot::propose_ballot(&mut gov_state.tracker, guid, t);
    }

    /// Check if a proposal has already been made for this transaction.
    fun is_unique_proposal<GovAction: drop + store>(tracker: &BallotTracker<TurnoutTally<GovAction>>, data: &GovAction): bool {
      // NOTE: Ballot.move does not check for duplicates. We need to check here.
      let list_pending = ballot::get_list_ballots_by_enum(tracker, ballot::get_pending_enum());

      let len = vector::length(list_pending);
      let i = 0;

      while (i < len) {
        let ballot = vector::borrow(list_pending, i);
        let ballot_data = ballot::get_type_struct(ballot);

        if (turnout_tally::get_tally_data(ballot_data) == data) return false;

        i = i + 1;
      };
      true
    }

}