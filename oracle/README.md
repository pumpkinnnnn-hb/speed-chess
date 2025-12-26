# Chess Betting Oracle

Stockfish-powered oracle service for Speed Chess Betting on Linera.

## Features

- Real-time position analysis using Stockfish engine
- Automatic odds calculation based on centipawn evaluation
- Monitors active games every 30 seconds
- Updates betting contract with current odds
- Graceful shutdown handling

## Architecture

```
oracle/
├── src/
│   ├── index.ts                    # Main entry point
│   ├── config.ts                   # Configuration
│   ├── scheduler/
│   │   └── gameScheduler.ts        # Cron-based game monitoring
│   ├── workers/
│   │   ├── stockfishEngine.ts      # Stockfish integration
│   │   └── gameMonitor.ts          # Game state polling
│   └── core/
│       ├── operations/
│       │   ├── updateOdds.ts       # Update betting odds
│       │   └── analyzePosition.ts  # Stockfish analysis
│       └── types/
│           └── index.ts            # TypeScript types
```

## How It Works

1. **Initialization**: Starts Stockfish engine with UCI protocol
2. **Game Discovery**: Queries game contract for active games
3. **Position Analysis**: Analyzes each position using Stockfish (depth 15)
4. **Odds Calculation**: Converts centipawn evaluation to win probabilities
5. **Blockchain Update**: Sends odds to betting contract via GraphQL mutation
6. **Continuous Monitoring**: Repeats every 30 seconds

## Odds Calculation Formula

```
Win Probability (White) = 1 / (1 + 10^(-eval/400))
Draw Probability = 0.25 * exp(-|eval|/5.0)
Win Probability (Black) = 1 - P(white) - P(draw)

Odds (basis points) = (1 / probability) * 10000
```

Clamped between 1.0x (10000) and 10.0x (100000).

## Environment Variables

Set by deployment script (`run.bash`):

- `GAME_APP_ID` - Game contract application ID
- `BETTING_APP_ID` - Betting contract application ID
- `LINERA_SERVICE_URL` - Linera GraphQL service URL
- `ORACLE_CHAIN_ID` - Oracle's chain ID

## Installation

```bash
npm install
```

## Build

```bash
npm run build
```

## Run

```bash
npm start
```

## Development

```bash
npm run dev
```

## Requirements

- Node.js 20+
- Stockfish binary installed (`apt-get install stockfish` or `brew install stockfish`)

## Configuration

Edit `src/config.ts`:

- `pollingInterval`: Game check frequency (default: 30000ms)
- `stockfishDepth`: Analysis depth (default: 15)

## Logging

Console output shows:
- Game monitoring cycles
- Position evaluations (centipawns)
- Calculated odds for White/Black/Draw
- GraphQL mutation success/failures

## Graceful Shutdown

Press Ctrl+C to stop. The oracle will:
1. Stop the scheduler
2. Quit Stockfish engine
3. Exit cleanly

## Error Handling

- Retries failed GraphQL requests
- Continues monitoring other games if one fails
- Logs all errors with context
- Removes stale games from cache

## Performance

- Depth 15 analysis: ~2 seconds per position
- Monitors up to 10+ concurrent games
- Memory usage: ~50MB + Stockfish (~100MB)

## Integration

The oracle integrates with:
- **Game Contract**: Queries for active games and positions
- **Betting Contract**: Sends UpdateOdds mutations

No authentication required (read-only queries + public mutations).
