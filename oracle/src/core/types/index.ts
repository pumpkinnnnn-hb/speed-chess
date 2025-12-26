export interface OracleConfig {
  gameAppId: string;
  bettingAppId: string;
  serviceUrl: string;
  chainId: string;
  pollingInterval: number; // milliseconds
  stockfishDepth: number;
}

export interface ChessGame {
  id: string;
  white_player: string;
  black_player: string;
  status: GameStatus;
  current_fen: string;
  move_count: number;
  result: GameResult | null;
  created_at: number;
}

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

export interface GameOdds {
  white_win: number; // Basis points (10000 = 1.0x)
  black_win: number;
  draw: number;
  evaluation: number; // Centipawn score
  last_updated: number;
}

export interface StockfishAnalysis {
  fen: string;
  evaluation: number; // Centipawns
  bestMove: string | null;
  depth: number;
  odds: {
    white_win: number;
    black_win: number;
    draw: number;
  };
}

export interface GameState {
  game: ChessGame;
  lastAnalysis: StockfishAnalysis | null;
  lastChecked: number;
}

export interface BetPool {
  game_id: string;
  total_pool: number;
  white_pool: number;
  black_pool: number;
  draw_pool: number;
  locked: boolean;
}
