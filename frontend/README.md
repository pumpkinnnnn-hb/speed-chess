# Speed Chess Betting - Frontend

Minimalist terminal-aesthetic frontend for live chess betting on Linera blockchain.

## Design Philosophy

Dark terminal/CLI aesthetic with neon green accents, featuring:
- Deep black/navy backgrounds (#0a0e1a, #0d1117)
- Neon green (#00ff9f) highlights and CTAs
- High contrast white text (#e6edf3)
- Monospace JetBrains Mono font throughout
- Subtle scan line effects
- Glowing neon borders on active elements
- Professional, minimalist spacing
- No shadows or gradients - pure terminal aesthetics

## Tech Stack

- **Framework**: Vite + React 18 + TypeScript
- **Styling**: TailwindCSS with custom terminal theme
- **State**: Zustand (global state)
- **Data Fetching**: React Query (GraphQL polling)
- **Chess Engine**: chess.js + react-chessboard
- **Animations**: Framer Motion
- **GraphQL**: graphql-request

## Features

### Live Chess Board
- Interactive chessboard with drag-and-drop moves
- Real-time game state updates (3s polling)
- Position evaluation bar
- Unicode chess pieces (♔♕♖♗♘♙)
- Algebraic notation labels

### Betting System
- Live odds display (White/Black/Draw)
- Quick bet amounts (10, 50, 100, MAX)
- Custom bet input
- Potential payout calculator
- Real-time bet pool visualization
- Progress bars for pool distribution

### Game Management
- Active games list sidebar
- Game selection and switching
- Game status indicators
- Move count and timestamp
- Player addresses (truncated)

### Wallet Integration
- Chain ID display
- Token balance
- Connected status indicator
- Address truncation (8...6 format)

## Installation

Install dependencies:
```bash
npm install
```

## Development

Start dev server:
```bash
npm run dev
```

Access at: http://localhost:5173

## Build

Build for production:
```bash
npm run build
```

Preview production build:
```bash
npm run preview
```

## Environment Variables

Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

Variables are automatically set by `run.bash` deployment script:
- `VITE_GAME_APP_ID` - Game contract application ID
- `VITE_BETTING_APP_ID` - Betting contract application ID
- `VITE_TOKEN_APP_ID` - Token contract application ID
- `VITE_LINERA_GRAPHQL_URL` - GraphQL endpoint (default: http://localhost:9001)
- `VITE_LINERA_FAUCET_URL` - Faucet URL (default: http://localhost:8080)

## Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── Header.tsx           # Top bar with wallet/balance
│   │   ├── ChessBoard.tsx       # Interactive chess board
│   │   ├── BettingPanel.tsx     # Odds and bet placement
│   │   └── GamesList.tsx        # Active games sidebar
│   ├── hooks/
│   │   └── useGames.ts          # React Query hooks
│   ├── lib/
│   │   └── graphql.ts           # GraphQL client & queries
│   ├── store/
│   │   └── gameStore.ts         # Zustand state management
│   ├── styles/
│   │   └── index.css            # TailwindCSS + custom styles
│   ├── types/
│   │   └── index.ts             # TypeScript types
│   ├── App.tsx                  # Main app component
│   └── main.tsx                 # Entry point
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

## GraphQL Integration

### Polling Intervals
- Active games: 5 seconds
- Current game: 3 seconds
- Game odds: 10 seconds
- Bet pools: 5 seconds

### Queries Used
- `activeGames` - Fetch all active games
- `game(gameId)` - Fetch specific game state
- `gameOdds(gameId)` - Fetch current betting odds
- `betPool(gameId)` - Fetch bet pool distribution

### Mutations Used
- `makeMove(gameId, from, to, promotion)` - Submit chess move
- `createGame(opponentChain)` - Create new game
- `placeBet(gameId, outcome, amount)` - Place bet

## Custom Theme

TailwindCSS extended with terminal colors:
```js
colors: {
  terminal: {
    bg: '#0a0e1a',           // Background
    surface: '#0d1117',      // Surface/card
    border: '#1f2937',       // Borders
    text: '#e6edf3',         // Text
    muted: '#8b949e',        // Muted text
    neon: '#00ff9f',         // Primary accent
    'neon-dim': '#00d084',   // Dimmed accent
    success: '#00ff9f',      // Success
    error: '#ff4757',        // Error
    warning: '#ffa502',      // Warning
  }
}
```

## Custom Styles

### Neon Border Effect
```css
.neon-border {
  border: 1px solid rgba(0, 255, 159, 0.5);
  box-shadow: 
    0 0 5px rgba(0, 255, 159, 0.3),
    inset 0 0 5px rgba(0, 255, 159, 0.1);
}

.neon-border:hover {
  border-color: #00ff9f;
  box-shadow: 
    0 0 10px rgba(0, 255, 159, 0.5),
    0 0 20px rgba(0, 255, 159, 0.3);
}
```

### Scan Line Effect
Background scan lines animate across screen:
```css
body::before {
  content: '';
  position: fixed;
  background: linear-gradient(
    rgba(0, 255, 159, 0.03) 50%,
    rgba(0, 0, 0, 0.05) 50%
  );
  background-size: 100% 4px;
  animation: scan 8s linear infinite;
}
```

### Terminal Button
```css
.terminal-button {
  @apply px-4 py-2 border border-terminal-border bg-terminal-surface;
  @apply hover:border-terminal-neon/50 hover:text-terminal-neon;
  @apply font-mono text-sm tracking-wide;
}

.terminal-button-primary {
  @apply border-terminal-neon bg-terminal-neon/10 text-terminal-neon;
  @apply hover:bg-terminal-neon/20;
}
```

## Performance

- Code splitting with dynamic imports
- React Query caching and deduplication
- Optimized re-renders with Zustand selectors
- Memoized expensive calculations
- Lazy loading for non-critical components

## Accessibility

- Keyboard navigation support
- ARIA labels on interactive elements
- High contrast color scheme (WCAG AAA)
- Focus indicators on all interactive elements
- Semantic HTML structure

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## Deployment

Frontend is deployed via Docker:
```bash
cd /build/frontend
npm install
npm run dev -- --host 0.0.0.0
```

Access at: http://localhost:5173

## Integration with Contracts

Frontend connects to three Linera contracts:
1. **Game Contract** - Chess game logic, moves, state
2. **Betting Contract** - Odds, pools, bet placement
3. **Token Contract** - Balance queries, transfers

All communication via GraphQL over HTTP.

## Development Tips

### Hot Module Replacement
Vite provides instant HMR - changes appear immediately without page reload.

### Type Safety
Full TypeScript coverage with strict mode enabled. All GraphQL responses are typed.

### State Management
Zustand provides simple, performant state:
```ts
const { activeGame, setActiveGame } = useGameStore();
```

### Data Fetching
React Query handles caching and polling:
```ts
const { data: games } = useActiveGames(); // Auto-polls every 5s
```

## Troubleshooting

**Issue**: Blank screen on load
- Check `.env` has correct app IDs
- Verify GraphQL endpoint is accessible
- Check browser console for errors

**Issue**: Moves not working
- Ensure game is in Active status
- Check if it's your turn to move
- Verify WebSocket connection

**Issue**: Odds not updating
- Check oracle service is running
- Verify polling intervals in useGames.ts
- Check GraphQL mutations are successful

## License

MIT
