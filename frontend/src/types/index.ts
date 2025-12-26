export enum GameStatus {
  Pending = 'Pending',
  Active = 'Active',
  Finished = 'Finished',
  Abandoned = 'Abandoned',
}

export enum GameResult {
  WhiteWins = 'WhiteWins',
  BlackWins = 'BlackWins',
  Draw = 'Draw',
}

export enum BetOutcome {
  White = 'White',
  Black = 'Black',
  Draw = 'Draw',
}

export interface ChessGame {
  id: string;
  whitePlayer: string;
  blackPlayer: string;
  status: GameStatus;
  currentFen: string;
  moveCount: number;
  result: GameResult | null;
  createdAt: number;
  updatedAt: number;
}

export interface GameOdds {
  whiteOdds: number; // Basis points (10000 = 1.0x)
  blackOdds: number;
  drawOdds: number;
}

export interface BetPool {
  gameId: string;
  totalPool: number;
  whitePool: number;
  blackPool: number;
  drawPool: number;
  isLocked: boolean;
}

export interface Bet {
  id: string;
  gameId: string;
  bettor: string;
  outcome: BetOutcome;
  amount: number;
  odds: number;
  potentialPayout: number;
  placedAt: number;
  settled: boolean;
  payout: number | null;
}

export interface WalletInfo {
  chainId: string;
  balance: number;
  isConnected: boolean;
}

export interface Move {
  from: string;
  to: string;
  promotion?: string;
}

export interface GraphQLResponse<T> {
  data?: T;
  errors?: Array<{ message: string }>;
}
