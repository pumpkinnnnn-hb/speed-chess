# Speed Chess Betting Contract

A production-ready betting contract for chess games on the Linera blockchain, implementing proportional payout pools, odds management, and automated settlement.

## Architecture Overview

This contract follows Linera SDK 0.15.6 patterns and implements a complete betting system with the following components:

### Core Components

1. **BettingState** - Persistent state management using LinEra Views
   - `pools`: MapView<String, BetPool> - All bet pools indexed by game_id
   - `bets`: MapView<String, BetRecord> - All bets indexed by bet_id
   - `game_bets`: MapView<String, Vec<String>> - Index of bets by game_id
   - `user_bets`: MapView<String, Vec<String>> - Index of bets by bettor
   - `odds`: MapView<String, GameOdds> - Current odds for each game
   - `game_results`: MapView<String, GameResult> - Settled game results
   - `claimed_winnings`: MapView<String, bool> - Double-claim prevention
   - `oracle_chain`: RegisterView<Option<ChainId>> - Oracle authorization
   - `platform_fee_bps`: RegisterView<u64> - Platform fee (basis points)
   - `min_bet_amount`: RegisterView<Amount> - Minimum bet amount

2. **Operations** - User-initiated actions
   - `PlaceBet` - Place a bet on a game outcome (WhiteWins/BlackWins/Draw)
   - `ClaimWinnings` - Claim proportional winnings from a won bet
   - `UpdateOdds` - Oracle-only operation to update game odds
   - `SettleGame` - Settle a game and mark bet statuses (Won/Lost)
   - `Initialize` - One-time setup with oracle address and parameters

3. **Messages** - Cross-contract communication from Game contract
   - `GameStarted` → Locks the bet pool (no more bets allowed)
   - `PositionUpdated` → Informational (oracle can update odds)
   - `GameFinished` → Stores result and automatically settles bets

## Security Features

### 1. Check-Effects-Interactions Pattern
```rust
// Mark as claimed BEFORE transfer to prevent reentrancy
self.state.claimed_winnings.insert(&bet_id, true).await
    .expect("Failed to mark as claimed");

// Then calculate and transfer winnings
let payout = calculate_payout(...)?;
```

### 2. Arithmetic Overflow Protection
All financial calculations use checked arithmetic:
```rust
pool.total_pool = pool.total_pool
    .checked_add(amount_u64)
    .ok_or(BettingError::ArithmeticOverflow)?;
```

### 3. Pool Locking
Pools are automatically locked when a game starts to prevent front-running:
```rust
if pool.locked {
    return Err(BettingError::PoolLocked);
}
```

### 4. Oracle Authorization
Only the designated oracle chain can update odds:
```rust
let oracle = self.state.oracle_chain.get()
    .ok_or(BettingError::NotInitialized)?;
// Verify caller matches oracle (implementation specific)
```

### 5. Input Validation
- Minimum bet amounts enforced
- Odds validation (sum must be 80-120% of 100%)
- Bet selection must be valid enum
- All amounts use `Amount` type (no raw integers)

## Payout Calculation

The contract implements proportional payout distribution:

```
Formula: payout = (bet_amount / winning_pool) * (total_pool - platform_fee)

Example:
- Total pool: 1000 tokens
- Winning pool (WhiteWins): 500 tokens
- User bet: 100 tokens on WhiteWins
- Platform fee: 1% (100 basis points)

Calculation:
- Fee = 1000 * 100 / 10000 = 10 tokens
- Pool after fee = 1000 - 10 = 990 tokens
- User payout = (100 / 500) * 990 = 198 tokens
```

### Edge Cases Handled:
1. **No winners**: If no one bet on the winning outcome, return original bet
2. **Overflow**: Uses u128 intermediate calculations, checked conversions
3. **Division by zero**: Explicitly handled before division

## Data Flow

### Bet Placement Flow
```
User → PlaceBet Operation
  ↓
Validate (not locked, amount >= min, pool exists)
  ↓
Update pool totals (white_pool/black_pool/draw_pool)
  ↓
Create BetRecord with current odds
  ↓
Update indexes (game_bets, user_bets)
  ↓
State persisted
```

### Game Settlement Flow
```
Game Contract → GameFinished Message
  ↓
Store game result in game_results
  ↓
Automatically call SettleGame
  ↓
Iterate all bets for game
  ↓
Update each bet status (Won/Lost)
  ↓
State persisted
```

### Winnings Claim Flow
```
User → ClaimWinnings Operation
  ↓
Check not already claimed
  ↓
Verify bet status == Won
  ↓
Calculate proportional payout
  ↓
Mark as claimed (BEFORE transfer)
  ↓
Transfer tokens to bettor
  ↓
State persisted
```

## GraphQL API

The service exposes a comprehensive GraphQL API for querying:

```graphql
type Query {
  # Get bet pool by game ID
  pool(game_id: String!): BetPool

  # Get all bet pools (paginate in production)
  pools: [BetPool!]!

  # Get specific bet by ID
  bet(bet_id: String!): BetRecord

  # Get all bets for a game
  gameBets(game_id: String!): [BetRecord!]!

  # Get all bets for a user
  userBets(bettor: String!): [BetRecord!]!

  # Get current odds for a game
  odds(game_id: String!): GameOdds

  # Get game result
  gameResult(game_id: String!): GameResult

  # Check if winnings claimed
  isClaimed(bet_id: String!): Boolean!

  # Get contract configuration
  config: BettingConfig!
}

type BettingConfig {
  oracleChain: String
  platformFeeBps: Int!
  minBetAmount: Int!
}
```

## Error Handling

All operations return typed errors:
- `PoolLocked` - Attempted to bet after game started
- `BetTooSmall` - Bet below minimum
- `PoolNotFound` - Invalid game ID
- `BetNotFound` - Invalid bet ID
- `AlreadyClaimed` - Double claim attempt
- `BetNotWon` - Tried to claim losing bet
- `UnauthorizedOracle` - Non-oracle updating odds
- `GameNotSettled` - Game result not available
- `InvalidOdds` - Odds validation failed
- `ArithmeticOverflow` - Calculation overflow
- `NotInitialized` - Contract not set up
- `AlreadyInitialized` - Re-initialization attempt
- `InsufficientBalance` - Not enough tokens

## Deployment & Initialization

### 1. Deploy Contract
```bash
linera project publish-and-create betting
```

### 2. Initialize
```rust
Operation::Initialize {
    oracle_chain: ChainId::from_str("..."),
    platform_fee_bps: 100, // 1%
    min_bet_amount: Amount::from_tokens(10),
}
```

### 3. Integration with Game Contract
The Game contract should send messages to this contract:
```rust
// When game starts
Message::GameStarted {
    game_id,
    white_player,
    black_player,
    timestamp,
}

// When game finishes
Message::GameFinished {
    game_id,
    result, // WhiteWins, BlackWins, or Draw
    timestamp,
}
```

## Testing

The contract includes comprehensive unit tests:

```bash
cargo test
```

Tests cover:
- Payout calculations (even split, no winners, overflow)
- Odds validation (valid ranges, invalid ranges)
- Bet ID generation
- State transitions
- Error conditions

## Future Enhancements

1. **Refund Mechanism** - For abandoned/cancelled games
2. **Partial Settlements** - Support for resignations/timeouts
3. **Live Betting** - In-game bet placement with dynamic odds
4. **Multi-Currency** - Support different token types
5. **Fee Distribution** - Revenue sharing for platform/validators
6. **Bet Limits** - Maximum bet sizes, daily limits
7. **Analytics** - Historical performance, pool statistics
8. **Pagination** - For large result sets in GraphQL
9. **Events** - Emit events for indexing/notifications
10. **Governance** - DAO control of parameters

## Dependencies

```toml
[dependencies]
chess-betting-abi = { path = "../abi" }
linera-sdk = "0.15.6"
linera-views = "0.15.6"
async-graphql = "7.0.17"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
```

## License

Copyright © 2025 Speed Chess Betting Platform

## Notes

- All timestamps are in microseconds since epoch
- Chain IDs are represented as hex strings
- Odds are in basis points (10000 = 100%)
- Amounts use Linera's `Amount` type
- State is persisted using LinEra Views (optimized for blockchain storage)

## Contact

For questions or contributions, please open an issue in the repository.
