#![cfg(test)]

extern crate std;

use super::*;
use soroban_sdk::{Env, String};


// ─────────────────────────────────────────────────────────────────────────────
// Helper: spin up a fresh Env and register the contract, returning the client.
// ─────────────────────────────────────────────────────────────────────────────
fn setup() -> (Env, PollRewardContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register_contract(None, PollRewardContract);
    let client = PollRewardContractClient::new(&env, &contract_id);
    (env, client)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: create a standard test poll and return its poll_id.
// ─────────────────────────────────────────────────────────────────────────────
fn create_test_poll(env: &Env, client: &PollRewardContractClient) -> u64 {
    client.create_poll(
        &String::from_str(env, "Best Blockchain?"),
        &String::from_str(env, "Stellar"),
        &String::from_str(env, "Ethereum"),
        &10_u64,
    )
}


// =============================================================================
// TEST GROUP 1 — create_poll
// =============================================================================

/// A freshly created poll should be returned with the correct fields.
#[test]
fn test_create_poll_returns_correct_poll_id() {
    let (env, client) = setup();

    let poll_id = create_test_poll(&env, &client);

    assert_eq!(poll_id, 1, "First poll should have ID 1");
}

/// Creating multiple polls should produce sequential, unique IDs.
#[test]
fn test_create_multiple_polls_sequential_ids() {
    let (env, client) = setup();

    let id1 = create_test_poll(&env, &client);
    let id2 = create_test_poll(&env, &client);
    let id3 = create_test_poll(&env, &client);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

/// After creation the poll should be active with zero vote counts.
#[test]
fn test_create_poll_initial_state() {
    let (env, client) = setup();

    let poll_id = create_test_poll(&env, &client);
    let poll = client.view_poll(&poll_id);

    assert_eq!(poll.poll_id, 1);
    assert_eq!(poll.votes_a, 0);
    assert_eq!(poll.votes_b, 0);
    assert_eq!(poll.reward_per_vote, 10);
    assert!(poll.is_active, "Newly created poll must be active");
}

/// Global stats should reflect the new poll after creation.
#[test]
fn test_create_poll_updates_global_stats() {
    let (env, client) = setup();

    create_test_poll(&env, &client);
    let stats = client.view_poll_stats();

    assert_eq!(stats.total_polls, 1);
    assert_eq!(stats.active_polls, 1);
    assert_eq!(stats.total_votes, 0);
    assert_eq!(stats.total_rewarded, 0);
}

/// Stats should correctly accumulate across multiple poll creations.
#[test]
fn test_create_poll_stats_accumulate() {
    let (env, client) = setup();

    create_test_poll(&env, &client);
    create_test_poll(&env, &client);
    create_test_poll(&env, &client);

    let stats = client.view_poll_stats();
    assert_eq!(stats.total_polls, 3);
    assert_eq!(stats.active_polls, 3);
}


// =============================================================================
// TEST GROUP 2 — cast_vote
// =============================================================================

/// Voting for Option A (choice = 1) should increment votes_a and return reward.
#[test]
fn test_cast_vote_option_a_increments_votes_a() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    let reward = client.cast_vote(&poll_id, &101_u64, &1_u64);
    let poll = client.view_poll(&poll_id);

    assert_eq!(reward, 10, "Reward should match reward_per_vote");
    assert_eq!(poll.votes_a, 1);
    assert_eq!(poll.votes_b, 0);
}

/// Voting for Option B (choice = 2) should increment votes_b and return reward.
#[test]
fn test_cast_vote_option_b_increments_votes_b() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    let reward = client.cast_vote(&poll_id, &202_u64, &2_u64);
    let poll = client.view_poll(&poll_id);

    assert_eq!(reward, 10);
    assert_eq!(poll.votes_a, 0);
    assert_eq!(poll.votes_b, 1);
}

/// Multiple distinct voters should all accumulate their votes correctly.
#[test]
fn test_cast_vote_multiple_voters() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.cast_vote(&poll_id, &1_u64, &1_u64); // voter 1 → A
    client.cast_vote(&poll_id, &2_u64, &1_u64); // voter 2 → A
    client.cast_vote(&poll_id, &3_u64, &2_u64); // voter 3 → B

    let poll = client.view_poll(&poll_id);
    assert_eq!(poll.votes_a, 2);
    assert_eq!(poll.votes_b, 1);
}

/// Global stats should be updated correctly after votes are cast.
#[test]
fn test_cast_vote_updates_global_stats() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.cast_vote(&poll_id, &1_u64, &1_u64);
    client.cast_vote(&poll_id, &2_u64, &2_u64);

    let stats = client.view_poll_stats();
    assert_eq!(stats.total_votes, 2);
    assert_eq!(stats.total_rewarded, 20); // 2 votes × 10 reward each
}

/// A voter should NOT be able to vote twice in the same poll — must panic.
#[test]
#[should_panic(expected = "You have already voted in this poll.")]
fn test_cast_vote_double_vote_panics() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.cast_vote(&poll_id, &42_u64, &1_u64); // first vote — OK
    client.cast_vote(&poll_id, &42_u64, &2_u64); // second vote — must panic
}

/// Passing an invalid choice (not 1 or 2) should panic.
#[test]
#[should_panic(expected = "Invalid choice! Must be 1 or 2.")]
fn test_cast_vote_invalid_choice_panics() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.cast_vote(&poll_id, &99_u64, &5_u64); // choice 5 is invalid
}

/// Voting on a non-existent poll (poll_id that was never created) should panic
/// because the returned default Poll has is_active = false.
#[test]
#[should_panic(expected = "Poll is closed. Cannot vote.")]
fn test_cast_vote_on_nonexistent_poll_panics() {
    let (env, client) = setup();

    // No poll was ever created; poll_id 999 does not exist.
    client.cast_vote(&999_u64, &1_u64, &1_u64);
}

/// Voting on a closed poll should panic.
#[test]
#[should_panic(expected = "Poll is closed. Cannot vote.")]
fn test_cast_vote_on_closed_poll_panics() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.close_poll(&poll_id);                   // close the poll first
    client.cast_vote(&poll_id, &1_u64, &1_u64);   // voting now must panic
}

/// A voter's vote record should be persisted correctly after casting.
#[test]
fn test_cast_vote_stores_vote_record() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.cast_vote(&poll_id, &55_u64, &2_u64);

    let record = client.view_vote_record(&poll_id, &55_u64);
    assert_eq!(record.poll_id, poll_id);
    assert_eq!(record.voter_id, 55);
    assert_eq!(record.choice, 2);
    assert_eq!(record.rewarded, 10);
}

/// Reward should scale with the reward_per_vote configured at poll creation.
#[test]
fn test_cast_vote_reward_reflects_custom_reward_per_vote() {
    let (env, client) = setup();

    // Create a poll with reward = 50
    let poll_id = client.create_poll(
        &String::from_str(&env, "Custom Reward Poll"),
        &String::from_str(&env, "Yes"),
        &String::from_str(&env, "No"),
        &50_u64,
    );

    let reward = client.cast_vote(&poll_id, &7_u64, &1_u64);
    assert_eq!(reward, 50);
}


// =============================================================================
// TEST GROUP 3 — close_poll
// =============================================================================

/// Closing an active poll should set is_active to false.
#[test]
fn test_close_poll_sets_inactive() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.close_poll(&poll_id);
    let poll = client.view_poll(&poll_id);

    assert!(!poll.is_active, "Poll should be inactive after close_poll");
}

/// Closing a poll should decrement active_polls in global stats.
#[test]
fn test_close_poll_decrements_active_polls_in_stats() {
    let (env, client) = setup();
    create_test_poll(&env, &client);
    let poll_id2 = create_test_poll(&env, &client);

    client.close_poll(&poll_id2);
    let stats = client.view_poll_stats();

    assert_eq!(stats.active_polls, 1, "Only one poll should remain active");
    assert_eq!(stats.total_polls, 2, "total_polls must not change on close");
}

/// Attempting to close an already-closed poll should panic.
#[test]
#[should_panic(expected = "Poll is already closed.")]
fn test_close_poll_already_closed_panics() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.close_poll(&poll_id); // first close — OK
    client.close_poll(&poll_id); // second close — must panic
}

/// Closing a poll that was never created (default is_active = false) should panic.
#[test]
#[should_panic(expected = "Poll is already closed.")]
fn test_close_poll_nonexistent_poll_panics() {
    let (_env, client) = setup();
    client.close_poll(&888_u64);
}


// =============================================================================
// TEST GROUP 4 — view_poll
// =============================================================================

/// Viewing a poll that does not exist should return the default Not_Found object.
#[test]
fn test_view_poll_not_found_returns_default() {
    let (env, client) = setup();
    let poll = client.view_poll(&999_u64);

    assert_eq!(poll.poll_id, 0);
    assert_eq!(poll.votes_a, 0);
    assert_eq!(poll.votes_b, 0);
    assert!(!poll.is_active);
    assert_eq!(poll.title, String::from_str(&env, "Not_Found"));
}

/// After votes are cast, view_poll should reflect the updated tally.
#[test]
fn test_view_poll_reflects_updated_vote_tally() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.cast_vote(&poll_id, &10_u64, &1_u64);
    client.cast_vote(&poll_id, &11_u64, &1_u64);
    client.cast_vote(&poll_id, &12_u64, &2_u64);

    let poll = client.view_poll(&poll_id);
    assert_eq!(poll.votes_a, 2);
    assert_eq!(poll.votes_b, 1);
}


// =============================================================================
// TEST GROUP 5 — view_poll_stats
// =============================================================================

/// Stats should start at all-zeros before any interaction.
#[test]
fn test_view_poll_stats_initial_zeros() {
    let (_env, client) = setup();
    let stats = client.view_poll_stats();

    assert_eq!(stats.total_polls, 0);
    assert_eq!(stats.active_polls, 0);
    assert_eq!(stats.total_votes, 0);
    assert_eq!(stats.total_rewarded, 0);
}

/// End-to-end: create, vote, close — stats must reflect every step correctly.
#[test]
fn test_view_poll_stats_full_lifecycle() {
    let (env, client) = setup();

    // Create two polls
    let p1 = create_test_poll(&env, &client);
    let p2 = create_test_poll(&env, &client);

    // Cast 3 votes across both polls (reward = 10 each)
    client.cast_vote(&p1, &1_u64, &1_u64);
    client.cast_vote(&p1, &2_u64, &2_u64);
    client.cast_vote(&p2, &3_u64, &1_u64);

    // Close one poll
    client.close_poll(&p1);

    let stats = client.view_poll_stats();
    assert_eq!(stats.total_polls, 2);
    assert_eq!(stats.active_polls, 1);
    assert_eq!(stats.total_votes, 3);
    assert_eq!(stats.total_rewarded, 30);
}


// =============================================================================
// TEST GROUP 6 — view_vote_record
// =============================================================================

/// Viewing a vote record that does not exist should return the default zero record.
#[test]
fn test_view_vote_record_not_found_returns_default() {
    let (_env, client) = setup();
    let record = client.view_vote_record(&1_u64, &999_u64);

    assert_eq!(record.poll_id, 0);
    assert_eq!(record.voter_id, 0);
    assert_eq!(record.choice, 0);
    assert_eq!(record.rewarded, 0);
}

/// Two different voters in the same poll should each have independent records.
#[test]
fn test_view_vote_record_independent_per_voter() {
    let (env, client) = setup();
    let poll_id = create_test_poll(&env, &client);

    client.cast_vote(&poll_id, &1_u64, &1_u64); // voter 1 → Option A
    client.cast_vote(&poll_id, &2_u64, &2_u64); // voter 2 → Option B

    let r1 = client.view_vote_record(&poll_id, &1_u64);
    let r2 = client.view_vote_record(&poll_id, &2_u64);

    assert_eq!(r1.choice, 1);
    assert_eq!(r2.choice, 2);
    assert_eq!(r1.rewarded, 10);
    assert_eq!(r2.rewarded, 10);
    assert_eq!(r1.voter_id, 1);
    assert_eq!(r2.voter_id, 2);
}

/// The same voter in two different polls should have two independent records.
#[test]
fn test_view_vote_record_same_voter_different_polls() {
    let (env, client) = setup();
    let p1 = create_test_poll(&env, &client);
    let p2 = create_test_poll(&env, &client);

    client.cast_vote(&p1, &77_u64, &1_u64);
    client.cast_vote(&p2, &77_u64, &2_u64);

    let r1 = client.view_vote_record(&p1, &77_u64);
    let r2 = client.view_vote_record(&p2, &77_u64);

    assert_eq!(r1.poll_id, p1);
    assert_eq!(r1.choice, 1);
    assert_eq!(r2.poll_id, p2);
    assert_eq!(r2.choice, 2);
}


// =============================================================================
// TEST GROUP 7 — end-to-end / integration scenarios
// =============================================================================

/// Full happy-path: create → vote → close → verify everything is consistent.
#[test]
fn test_full_happy_path() {
    let (env, client) = setup();

    // 1. Create poll
    let poll_id = client.create_poll(
        &String::from_str(&env, "Favourite Language?"),
        &String::from_str(&env, "Rust"),
        &String::from_str(&env, "Go"),
        &25_u64,
    );
    assert_eq!(poll_id, 1);

    // 2. Two voters vote
    let r1 = client.cast_vote(&poll_id, &10_u64, &1_u64);
    let r2 = client.cast_vote(&poll_id, &20_u64, &2_u64);
    assert_eq!(r1, 25);
    assert_eq!(r2, 25);

    // 3. Verify poll tally
    let poll = client.view_poll(&poll_id);
    assert_eq!(poll.votes_a, 1);
    assert_eq!(poll.votes_b, 1);
    assert!(poll.is_active);

    // 4. Close the poll
    client.close_poll(&poll_id);
    let closed = client.view_poll(&poll_id);
    assert!(!closed.is_active);

    // 5. Verify global stats
    let stats = client.view_poll_stats();
    assert_eq!(stats.total_polls, 1);
    assert_eq!(stats.active_polls, 0);
    assert_eq!(stats.total_votes, 2);
    assert_eq!(stats.total_rewarded, 50);
}

/// Zero reward poll: votes should still be recorded; reward returned should be 0.
#[test]
fn test_zero_reward_poll() {
    let (env, client) = setup();

    let poll_id = client.create_poll(
        &String::from_str(&env, "Zero Reward Poll"),
        &String::from_str(&env, "Yes"),
        &String::from_str(&env, "No"),
        &0_u64,
    );

    let reward = client.cast_vote(&poll_id, &5_u64, &1_u64);
    assert_eq!(reward, 0);

    let stats = client.view_poll_stats();
    assert_eq!(stats.total_rewarded, 0);
    assert_eq!(stats.total_votes, 1);
}

/// A voter who participates in multiple polls should have independent records
/// and the global total_votes should count every vote across all polls.
#[test]
fn test_voter_participates_in_multiple_polls() {
    let (env, client) = setup();

    let p1 = create_test_poll(&env, &client);
    let p2 = create_test_poll(&env, &client);
    let p3 = create_test_poll(&env, &client);

    // Voter 99 votes in all three polls
    client.cast_vote(&p1, &99_u64, &1_u64);
    client.cast_vote(&p2, &99_u64, &2_u64);
    client.cast_vote(&p3, &99_u64, &1_u64);

    let stats = client.view_poll_stats();
    assert_eq!(stats.total_votes, 3);
    assert_eq!(stats.total_rewarded, 30); // 3 × 10
}