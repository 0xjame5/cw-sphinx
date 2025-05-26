# CW-Sphinx: A CosmWasm Lottery Contract

CW-Sphinx is a decentralized lottery contract built on CosmWasm, allowing users to participate in a fair and transparent lottery system. The contract manages ticket purchases, winner selection, and prize distribution with a configurable house fee.

## Features

- **Ticket System**: Users can purchase multiple tickets for a fixed price
- **Configurable Duration**: Lottery rounds have a set duration after which no more tickets can be purchased
- **Fair Winner Selection**: Uses a seeded random number generator for transparent winner selection
- **House Fee**: Configurable percentage of the prize pool goes to the contract admin
- **State Management**: Clear lottery states (OPEN, CHOOSING, CLOSED) for proper flow control
- **Prize Distribution**: Automatic distribution of prizes to winners and house fees to admin

## Contract States

The lottery operates in three distinct states:

1. **OPEN**: The lottery is accepting ticket purchases
2. **CHOOSING**: The lottery period has ended, and a winner is being selected
3. **CLOSED**: A winner has been selected and prizes can be claimed

## Messages

### Instantiation
```rust
InstantiateMsg {
    ticket_cost: Coin,        // Cost per ticket
    lottery_duration: Duration, // How long the lottery runs
    admin: String,            // Admin address
    house_fee: u64,          // House fee percentage
}
```

### Execution Messages
- `BuyTicket { num_tickets: u64 }`: Purchase lottery tickets
- `ExecuteLottery { seed: u64 }`: Select a winner (admin only)
- `ClaimTokens {}`: Claim lottery winnings

### Query Messages
- `TicketCount { addr: Addr }`: Check number of tickets for an address
- `LotteryState {}`: Get current lottery state and total tickets
- `Config {}`: View contract configuration

## Usage Flow

1. **Initialization**: Deploy the contract with initial parameters
2. **Ticket Sales**: Users can buy tickets while the lottery is OPEN
3. **Winner Selection**: Admin executes the lottery with a seed after the duration expires
4. **Prize Claim**: Winner claims their prize, with house fee going to admin

## Current Randomness Implementation

The current version of the contract uses an admin-based system for randomness:
- The admin provides a seed value when executing the lottery
- This seed is used to generate a random number for winner selection
- While this provides transparency (the seed is visible on-chain), it still requires trust in the admin

## Planned Improvements

A future version will integrate with Secret Network to achieve true decentralization:
- The contract will make a cross-chain call to Secret Network
- Secret Network will provide a verifiable random hash
- This hash will be used as the seed for winner selection
- This improvement will eliminate the need for an admin to provide the seed
- The process will be fully automated and trustless

## Security Features

- Admin-only winner selection (to be replaced with Secret Network integration)
- One-time prize claiming
- Proper state transitions
- Exact payment validation
- Fair random number generation
