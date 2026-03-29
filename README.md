# Poll Reward

---

## Project Title

**Poll Reward** — A Decentralized On-Chain Polling & Incentive System Built on Stellar using Soroban SDK

---

## Project Description

Poll Reward is a blockchain-based smart contract application built on the **Stellar network** using the **Soroban SDK**. It enables anyone to create transparent, tamper-proof polls on-chain and automatically rewards participants with tokens for casting their votes.

The contract eliminates the need for centralized poll management platforms and removes the possibility of result manipulation. Every vote and every reward distribution is recorded immutably on the Stellar ledger. Voters are incentivized to participate through a built-in reward mechanism — each valid vote earns the voter a predefined number of reward tokens, making community engagement both fair and financially rewarding.

This project is ideal for DAOs, community governance systems, product feedback loops, and any use case where trustless, incentivized opinion collection is required.

---

## Project Vision

> _"Make every voice count — and reward it."_

The vision behind Poll Reward is to solve two fundamental problems in traditional polling systems:

1. **Lack of trust** — Centralized polls can be manipulated, results can be altered, and participation cannot be verified.
2. **Lack of incentive** — People have little motivation to participate in polls, leading to low engagement and unrepresentative results.

By leveraging the immutability and transparency of the Stellar blockchain through Soroban smart contracts, Poll Reward aims to become the go-to infrastructure layer for **honest, high-participation, incentivized community polling** — from small DAOs to large-scale governance systems.

---

## Key Features

| Feature                          | Description                                                                                                        |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| 🗳️ **On-Chain Poll Creation**    | Any authorized user can create a poll with a title, two voting options, and a configurable reward per vote.        |
| 💰 **Automatic Token Rewards**   | Voters are automatically rewarded with tokens upon casting a valid vote — no manual distribution needed.           |
| 🔒 **Double-Vote Prevention**    | The contract enforces one-vote-per-voter-per-poll using a unique `(poll_id, voter_id)` mapping stored on-chain.    |
| 📊 **Live Poll Statistics**      | Global stats including total polls, active polls, total votes, and total rewards distributed are always queryable. |
| ✅ **Poll Lifecycle Management** | Admins can open and close polls, giving full control over the polling period.                                      |
| 🔍 **Transparent Vote Records**  | Every vote record (poll ID, voter ID, choice, reward, timestamp) is stored on the ledger and publicly verifiable.  |

---

## Smart Contract Functions

The contract exposes **4 core functions** and **2 view functions**:

### Core Functions

#### `create_poll(env, title, option_a, option_b, reward_per_vote) → u64`

Creates a new poll on-chain with two voting options and a reward amount per vote. Returns the unique `poll_id` of the newly created poll.

- Increments the global poll counter
- Marks the poll as **active**
- Records the creation timestamp from the ledger
- Updates global `PollStats`

---

#### `cast_vote(env, poll_id, voter_id, choice) → u64`

Allows a voter to cast their vote (choice: `1` = Option A, `2` = Option B) on an active poll. Returns the number of reward tokens earned.

- Validates that the poll is **active**
- Validates the **choice** (must be 1 or 2)
- Prevents **double-voting** using on-chain vote records
- Distributes reward tokens to the voter automatically
- Updates vote tallies and global statistics

---

#### `close_poll(env, poll_id)`

Closes an active poll, preventing any further votes from being cast.

- Sets `is_active` to `false`
- Decrements the active poll count in `PollStats`
- Emits a log event confirming closure

---

#### `view_poll(env, poll_id) → Poll`

Returns the full details of a poll including title, options, vote counts, reward per vote, status, and creation time. Returns a default `Not_Found` object if the poll does not exist.

---

### View / Query Functions

#### `view_poll_stats(env) → PollStats`

Returns the global statistics object containing:

- `total_polls` — All polls ever created
- `active_polls` — Currently open polls
- `total_votes` — All votes ever cast
- `total_rewarded` — Sum of all reward tokens distributed

---

#### `view_vote_record(env, poll_id, voter_id) → VoteRecord`

Returns a specific voter's vote record for a given poll, including their choice, reward received, and timestamp of vote. Returns a default zero-value record if no vote exists.

---

## Data Structures

```rust
pub struct Poll {
    pub poll_id: u64,
    pub title: String,
    pub option_a: String,
    pub option_b: String,
    pub votes_a: u64,
    pub votes_b: u64,
    pub reward_per_vote: u64,
    pub is_active: bool,
    pub created_at: u64,
}

pub struct VoteRecord {
    pub poll_id: u64,
    pub voter_id: u64,
    pub choice: u64,      // 1 = Option A, 2 = Option B
    pub rewarded: u64,
    pub voted_at: u64,
}

pub struct PollStats {
    pub total_polls: u64,
    pub active_polls: u64,
    pub total_votes: u64,
    pub total_rewarded: u64,
}
```

---

## Future Scope

The current contract lays a minimal but solid foundation. The following enhancements are planned for future iterations:

| Roadmap Item                       | Description                                                                                                                       |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| 🏆 **Multi-Option Polls**          | Extend polls to support more than 2 options (e.g., up to 6 choices) for richer data collection.                                   |
| ⏰ **Time-Locked Polls**           | Automatically close polls after a configurable deadline using ledger timestamps — no manual intervention needed.                  |
| 🪙 **Native Token Integration**    | Integrate with Stellar's native asset layer or a custom SAC (Stellar Asset Contract) to issue real on-chain tokens as rewards.    |
| 🧑‍⚖️ **Role-Based Access Control**   | Introduce admin roles so only whitelisted addresses can create or close polls.                                                    |
| 📈 **Weighted Voting**             | Allow votes to carry different weights based on token holdings or reputation scores.                                              |
| 🌐 **Frontend dApp**               | Build a React/Next.js frontend that interacts with this contract via the Soroban JS SDK for a full end-to-end polling experience. |
| 🔗 **Cross-Contract Reward Hooks** | Enable reward tokens to automatically trigger staking, NFT minting, or other DeFi actions upon a successful vote.                 |
| 🗃️ **Poll Archiving**              | Implement an archive mechanism that moves expired polls to cold storage to reduce ledger footprint and costs.                     |

---

## Getting Started

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- Stellar CLI (`stellar`)
- Soroban SDK

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/poll_reward_contract.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

### Invoke Example

```bash
# Create a poll
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- create_poll \
  --title "Best Blockchain?" \
  --option_a "Stellar" \
  --option_b "Ethereum" \
  --reward_per_vote 10

# Cast a vote (voter 42, choosing Option A)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- cast_vote \
  --poll_id 1 \
  --voter_id 42 \
  --choice 1
```

---

## Frontend

The project includes a modern, minimalistic React/Next.js frontend that provides an intuitive interface for interacting with the Poll Rewards smart contract.

### Frontend Features

- **📊 Statistics Dashboard**: View real-time poll metrics (total polls, active polls, total votes, rewards distributed)
- **🗳️ Poll Creation**: Create new polls with custom titles, two options, and reward amounts
- **🎯 Voting Interface**: Cast votes in active polls with instant reward notifications
- **📱 Responsive Design**: Modern UI that works seamlessly on desktop, tablet, and mobile
- **🔗 Wallet Integration**: Freighter wallet support for secure Stellar transactions
- **✨ Real-time Updates**: Live transaction status and error feedback

### Frontend Stack

- **Next.js 14** - React framework with App Router
- **TypeScript** - Type-safe development
- **Tailwind CSS** - Utility-first styling
- **@stellar/stellar-sdk** - Stellar blockchain integration
- **@stellar/freighter-api** - Wallet connection and signing

### Getting Started with Frontend

1. **Navigate to client directory**:

   ```bash
   cd client
   ```

2. **Install dependencies**:

   ```bash
   npm install
   ```

3. **Start development server**:

   ```bash
   npm run dev
   ```

4. **Open browser**:
   - Visit [http://localhost:3000](http://localhost:3000)
   - Connect your Freighter wallet
   - Start creating and voting in polls!

### Frontend Components

- **Statistics Tab**: Global poll metrics and blockchain activity
- **Create Poll Tab**: Form to set up new polls with rewards
- **Vote Tab**: Select poll ID and vote for Option A or B
- **Close Poll Tab**: Admin function to end voting periods

### Frontend Structure

```
client/
├── app/              # Next.js App Router pages
├── components/       # React components
│   ├── Contract.tsx  # Main poll interaction UI
│   ├── Navbar.tsx    # Navigation and wallet connection
│   └── ui/           # Reusable UI components
├── hooks/
│   └── contract.ts   # Soroban contract interaction helpers
├── lib/              # Utility functions
└── public/           # Static assets
```

---

## License

This project is open-source and available under the [MIT License](LICENSE).

## Contract deployment details

Contract id: 38e5741833640746c8c769af343b7ba8b546f097dbba80b6e8864f5622b9b952
Contract screenshot:<img width="1859" height="959" alt="image" src="https://github.com/user-attachments/assets/8b8b527e-efd0-48d4-b75d-fc57b6a7b885" />
## screenshot frontend
<img width="1919" height="874" alt="image" src="https://github.com/user-attachments/assets/c62d7eea-9f21-4bcb-9770-fd4ad351c408" />
<img width="1824" height="843" alt="image" src="https://github.com/user-attachments/assets/290e5cec-7858-4d1e-a6e0-6d6e9693ab95" />
<img width="1919" height="848" alt="image" src="https://github.com/user-attachments/assets/1dab1ea0-5e20-499c-a83a-3c5e930c31a4" />

