# Poll Rewards Frontend

A modern React frontend for the Poll Rewards smart contract on Stellar blockchain.

## Features

- **Create Polls**: Set up decentralized polls with two options and reward tokens
- **Vote & Earn**: Cast votes in active polls and automatically receive reward tokens
- **Live Statistics**: View global poll metrics and activity
- **Wallet Integration**: Connect with Freighter wallet for Stellar
- **Responsive Design**: Beautiful UI with animations and dark theme

## Getting Started

1. Install dependencies:

```bash
npm install
```

2. Start the development server:

```bash
npm run dev
```

3. Open [http://localhost:3000](http://localhost:3000) in your browser

4. Connect your Freighter wallet and start creating polls!

## Smart Contract Integration

This frontend interacts with the Poll Rewards contract deployed on Stellar testnet:

- **Create Poll**: `create_poll(title, option_a, option_b, reward_per_vote)`
- **Cast Vote**: `cast_vote(poll_id, voter_id, choice)` - earns reward tokens
- **Close Poll**: `close_poll(poll_id)` - admin function
- **View Stats**: `view_poll_stats()` - global statistics
- **View Poll**: `view_poll(poll_id)` - poll details

## Tech Stack

- **Next.js 14** - React framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling
- **Stellar SDK** - Blockchain integration
- **Freighter Wallet** - Wallet connection
- **Soroban** - Smart contract platform
