#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};


// Tracks global poll statistics
#[contracttype]
#[derive(Clone)]
pub struct PollStats {
    pub total_polls: u64,    // Total polls ever created
    pub active_polls: u64,   // Currently active polls
    pub total_votes: u64,    // Total votes cast across all polls
    pub total_rewarded: u64, // Total reward tokens distributed
}

// Symbol key for global stats
const POLL_STATS: Symbol = symbol_short!("P_STATS");

// Symbol key for reward token balance pool
const REWARD_POOL: Symbol = symbol_short!("RWD_POOL");

// Counter for unique poll IDs
const COUNT_POLL: Symbol = symbol_short!("C_POLL");

// Mapping poll_id -> Poll
#[contracttype]
pub enum Pollbook {
    Poll(u64),
}

// Mapping (poll_id, voter_id) -> VoteRecord
#[contracttype]
pub enum Votebook {
    Vote(u64, u64),
}


// Represents a single poll
#[contracttype]
#[derive(Clone)]
pub struct Poll {
    pub poll_id: u64,
    pub title: String,
    pub option_a: String,     // First option label
    pub option_b: String,     // Second option label
    pub votes_a: u64,         // Votes cast for option A
    pub votes_b: u64,         // Votes cast for option B
    pub reward_per_vote: u64, // Reward tokens given per valid vote
    pub is_active: bool,      // Whether the poll is still open
    pub created_at: u64,      // Ledger timestamp at creation
}


// Tracks whether a specific voter has voted in a specific poll, and what they chose
#[contracttype]
#[derive(Clone)]
pub struct VoteRecord {
    pub poll_id: u64,
    pub voter_id: u64,
    pub choice: u64,     // 1 = Option A, 2 = Option B
    pub rewarded: u64,   // Amount of reward tokens received
    pub voted_at: u64,   // Ledger timestamp of vote
}


#[contract]
pub struct PollRewardContract;

#[contractimpl]
impl PollRewardContract {

    // -------------------------------------------------------------------------
    // FUNCTION 1: create_poll
    // Admin/organizer creates a new poll with two options and a reward-per-vote.
    // Returns the unique poll_id of the newly created poll.
    // -------------------------------------------------------------------------
    pub fn create_poll(
        env: Env,
        title: String,
        option_a: String,
        option_b: String,
        reward_per_vote: u64,
    ) -> u64 {
        let mut count: u64 = env.storage().instance().get(&COUNT_POLL).unwrap_or(0);
        count += 1;

        let time = env.ledger().timestamp();

        let poll = Poll {
            poll_id: count,
            title,
            option_a,
            option_b,
            votes_a: 0,
            votes_b: 0,
            reward_per_vote,
            is_active: true,
            created_at: time,
        };

        // Persist the new poll
        env.storage().instance().set(&Pollbook::Poll(count), &poll);

        // Update global stats
        let mut stats = Self::view_poll_stats(env.clone());
        stats.total_polls += 1;
        stats.active_polls += 1;
        env.storage().instance().set(&POLL_STATS, &stats);

        // Persist new count
        env.storage().instance().set(&COUNT_POLL, &count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Poll created with ID: {}", count);
        count
    }


    // -------------------------------------------------------------------------
    // FUNCTION 2: cast_vote
    // A voter casts their vote on an active poll by providing:
    //   - poll_id : which poll they are voting in
    //   - voter_id: unique identifier for the voter (wallet address abstracted as u64)
    //   - choice  : 1 for Option A, 2 for Option B
    // On success the voter is automatically rewarded with tokens.
    // -------------------------------------------------------------------------
    pub fn cast_vote(env: Env, poll_id: u64, voter_id: u64, choice: u64) -> u64 {
        // Validate choice
        if choice != 1 && choice != 2 {
            log!(&env, "Invalid choice! Use 1 for Option A or 2 for Option B.");
            panic!("Invalid choice! Must be 1 or 2.");
        }

        // Load the poll
        let mut poll = Self::view_poll(env.clone(), poll_id.clone());

        // Ensure poll is active
        if !poll.is_active {
            log!(&env, "Poll-ID: {} is no longer active.", poll_id);
            panic!("Poll is closed. Cannot vote.");
        }

        // Ensure this voter has NOT already voted in this poll
        let vote_key = Votebook::Vote(poll_id.clone(), voter_id.clone());
        let existing: Option<VoteRecord> = env.storage().instance().get(&vote_key);
        if existing.is_some() {
            log!(&env, "Voter {} has already voted in Poll {}.", voter_id, poll_id);
            panic!("You have already voted in this poll.");
        }

        // Record the vote
        if choice == 1 {
            poll.votes_a += 1;
        } else {
            poll.votes_b += 1;
        }

        // Calculate reward
        let reward = poll.reward_per_vote;

        // Save updated poll
        env.storage().instance().set(&Pollbook::Poll(poll_id.clone()), &poll);

        // Save vote record
        let time = env.ledger().timestamp();
        let vote_record = VoteRecord {
            poll_id: poll_id.clone(),
            voter_id: voter_id.clone(),
            choice,
            rewarded: reward,
            voted_at: time,
        };
        env.storage().instance().set(&vote_key, &vote_record);

        // Update global stats
        let mut stats = Self::view_poll_stats(env.clone());
        stats.total_votes += 1;
        stats.total_rewarded += reward;
        env.storage().instance().set(&POLL_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Voter {} voted in Poll {} and earned {} reward tokens.", voter_id, poll_id, reward);
        reward // Return reward amount earned
    }


    // -------------------------------------------------------------------------
    // FUNCTION 3: close_poll
    // Admin closes an active poll, preventing further votes.
    // -------------------------------------------------------------------------
    pub fn close_poll(env: Env, poll_id: u64) {
        let mut poll = Self::view_poll(env.clone(), poll_id.clone());

        if !poll.is_active {
            log!(&env, "Poll-ID: {} is already closed.", poll_id);
            panic!("Poll is already closed.");
        }

        poll.is_active = false;
        env.storage().instance().set(&Pollbook::Poll(poll_id.clone()), &poll);

        // Update global stats
        let mut stats = Self::view_poll_stats(env.clone());
        if stats.active_polls > 0 {
            stats.active_polls -= 1;
        }
        env.storage().instance().set(&POLL_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Poll-ID: {} has been closed.", poll_id);
    }


    // -------------------------------------------------------------------------
    // FUNCTION 4: view_poll
    // Returns the full details of a poll by its poll_id.
    // Returns a default Poll object if poll not found.
    // -------------------------------------------------------------------------
    pub fn view_poll(env: Env, poll_id: u64) -> Poll {
        let key = Pollbook::Poll(poll_id);
        env.storage().instance().get(&key).unwrap_or(Poll {
            poll_id: 0,
            title: String::from_str(&env, "Not_Found"),
            option_a: String::from_str(&env, "Not_Found"),
            option_b: String::from_str(&env, "Not_Found"),
            votes_a: 0,
            votes_b: 0,
            reward_per_vote: 0,
            is_active: false,
            created_at: 0,
        })
    }


    // -------------------------------------------------------------------------
    // VIEW: view_poll_stats
    // Returns global statistics — total polls, active polls, votes, and rewards.
    // -------------------------------------------------------------------------
    pub fn view_poll_stats(env: Env) -> PollStats {
        env.storage().instance().get(&POLL_STATS).unwrap_or(PollStats {
            total_polls: 0,
            active_polls: 0,
            total_votes: 0,
            total_rewarded: 0,
        })
    }


    // -------------------------------------------------------------------------
    // VIEW: view_vote_record
    // Returns a voter's vote record for a specific poll.
    // Useful to confirm participation and reward earned.
    // -------------------------------------------------------------------------
    pub fn view_vote_record(env: Env, poll_id: u64, voter_id: u64) -> VoteRecord {
        let key = Votebook::Vote(poll_id.clone(), voter_id.clone());
        env.storage().instance().get(&key).unwrap_or(VoteRecord {
            poll_id: 0,
            voter_id: 0,
            choice: 0,
            rewarded: 0,
            voted_at: 0,
        })
    }
}