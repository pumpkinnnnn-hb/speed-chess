import { create } from 'zustand';
import { ChessGame, GameOdds, BetPool, Bet, WalletInfo } from '@/types';

interface GameState {
  // Games
  games: ChessGame[];
  activeGameId: string | null;
  activeGame: ChessGame | null;
  
  // Odds and pools
  odds: Record<string, GameOdds>;
  pools: Record<string, BetPool>;
  
  // Bets
  userBets: Bet[];
  
  // Wallet
  wallet: WalletInfo;
  
  // UI State
  isBettingPanelOpen: boolean;
  selectedBetOutcome: 'White' | 'Black' | 'Draw' | null;
  betAmount: string;
  
  // Actions
  setGames: (games: ChessGame[]) => void;
  setActiveGame: (gameId: string | null) => void;
  updateGame: (game: ChessGame) => void;
  setOdds: (gameId: string, odds: GameOdds) => void;
  setPool: (gameId: string, pool: BetPool) => void;
  addBet: (bet: Bet) => void;
  setWallet: (wallet: Partial<WalletInfo>) => void;
  setBettingPanel: (isOpen: boolean) => void;
  setSelectedBetOutcome: (outcome: 'White' | 'Black' | 'Draw' | null) => void;
  setBetAmount: (amount: string) => void;
  reset: () => void;
}

const initialWallet: WalletInfo = {
  chainId: '',
  balance: 0,
  isConnected: false,
};

export const useGameStore = create<GameState>((set, get) => ({
  games: [],
  activeGameId: null,
  activeGame: null,
  odds: {},
  pools: {},
  userBets: [],
  wallet: initialWallet,
  isBettingPanelOpen: false,
  selectedBetOutcome: null,
  betAmount: '',
  
  setGames: (games) => set({ games }),
  
  setActiveGame: (gameId) => {
    const activeGame = gameId 
      ? get().games.find(g => g.id === gameId) || null 
      : null;
    set({ activeGameId: gameId, activeGame });
  },
  
  updateGame: (game) => set((state) => {
    const games = state.games.map(g => g.id === game.id ? game : g);
    const activeGame = state.activeGameId === game.id ? game : state.activeGame;
    return { games, activeGame };
  }),
  
  setOdds: (gameId, odds) => set((state) => ({
    odds: { ...state.odds, [gameId]: odds }
  })),
  
  setPool: (gameId, pool) => set((state) => ({
    pools: { ...state.pools, [gameId]: pool }
  })),
  
  addBet: (bet) => set((state) => ({
    userBets: [...state.userBets, bet]
  })),
  
  setWallet: (wallet) => set((state) => ({
    wallet: { ...state.wallet, ...wallet }
  })),
  
  setBettingPanel: (isOpen) => set({ isBettingPanelOpen: isOpen }),
  
  setSelectedBetOutcome: (outcome) => set({ selectedBetOutcome: outcome }),
  
  setBetAmount: (amount) => set({ betAmount: amount }),
  
  reset: () => set({
    games: [],
    activeGameId: null,
    activeGame: null,
    odds: {},
    pools: {},
    userBets: [],
    selectedBetOutcome: null,
    betAmount: '',
  }),
}));
