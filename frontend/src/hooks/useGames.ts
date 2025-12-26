import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getGameClient, getBettingClient, GET_ACTIVE_GAMES, GET_GAME, GET_GAME_ODDS, GET_BET_POOL, MAKE_MOVE, CREATE_GAME, PLACE_BET } from '@/lib/graphql';
import { ChessGame, GameOdds, BetPool } from '@/types';

// Helper to get user's chain ID from localStorage with fallback
const getUserChainId = (): string => {
  return localStorage.getItem('linera_chain_id') || import.meta.env.VITE_CHAIN_ID || '';
};

export function useActiveGames() {
  return useQuery({
    queryKey: ['activeGames'],
    queryFn: async () => {
      console.log('🔄 [V5-MULTIPLAYER] Fetching games...');
      // Query from USER's current chain (respects localStorage)
      const chainId = getUserChainId();
      const appId = import.meta.env.VITE_GAME_APP_ID || '';
      const url = `${import.meta.env.VITE_LINERA_GRAPHQL_URL}/chains/${chainId}/applications/${appId}`;
      console.log('📡 Querying games from USER chain:', chainId.slice(0, 10) + '...');

      // ProcessInbox handled automatically - no manual call needed

      // Now query games (inbox is fresh)
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query: GET_ACTIVE_GAMES })
      });

      const json = await response.json();

      // Linera wraps data in "error" field as JSON string
      if (json.error && typeof json.error === 'string') {
        try {
          const parsed = JSON.parse(json.error);
          if (parsed.data?.allGames) {
            console.log('✅ [V5-MULTIPLAYER] Games from error wrapper:', parsed.data.allGames.length);
            return parsed.data.allGames;
          }
        } catch (e) {
          console.error('Failed to parse error field:', e);
        }
      }

      // Standard format (fallback)
      if (json.data?.allGames) {
        console.log('✅ [V5-MULTIPLAYER] Games from standard format:', json.data.allGames.length);
        return json.data.allGames;
      }

      console.error('❌ No games found in response');
      return [];
    },
    refetchInterval: 5000,
  });
}

export function useGame(gameId: string | null) {
  return useQuery({
    queryKey: ['game', gameId],
    queryFn: async () => {
      if (!gameId) return null;

      console.log('🎮 [ACTIVE GAME] Fetching game:', gameId);

      // Process inbox before querying specific game
      const chainId = getUserChainId();
      const appId = import.meta.env.VITE_GAME_APP_ID || '';
      const url = `${import.meta.env.VITE_LINERA_GRAPHQL_URL}/chains/${chainId}/applications/${appId}`;

      // ProcessInbox handled automatically by Linera service

      // CRITICAL FIX: The game(gameId) query crashes with "unreachable" error in Wasm!
      // Use allGames and filter client-side instead
      console.log('🔍 [ACTIVE GAME] Querying allGames and filtering for:', gameId);

      try {
        const response = await fetch(url, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query: GET_ACTIVE_GAMES })
        });

        const json = await response.json();
        console.log('📦 [ACTIVE GAME] Raw response:', JSON.stringify(json).slice(0, 200));

        // Handle Linera's error wrapper format
        let games: ChessGame[] = [];

        if (json.error && typeof json.error === 'string') {
          try {
            const parsed = JSON.parse(json.error);
            if (parsed.data?.allGames) {
              games = parsed.data.allGames;
            }
          } catch (e) {
            console.error('❌ [ACTIVE GAME] Failed to parse error field:', e);
          }
        } else if (json.data?.allGames) {
          games = json.data.allGames;
        }

        // Find the specific game
        const game = games.find((g: ChessGame) => g.id === gameId);

        if (game) {
          console.log('♟️ [ACTIVE GAME] Game state - moveCount:', game.moveCount, 'status:', game.status, 'FEN:', game.currentFen?.slice(0, 30) + '...');
          console.log('👥 [ACTIVE GAME] Players - White:', game.whitePlayer?.slice(0, 12), 'Black:', game.blackPlayer?.slice(0, 12));
          console.log('🎯 [ACTIVE GAME] Wallet chainId:', chainId.slice(0, 12));
          return game;
        } else {
          console.error('❌ [ACTIVE GAME] Game not found in allGames! Looking for:', gameId, 'Found games:', games.map(g => g.id));
          return null;
        }
      } catch (error) {
        console.error('❌ [ACTIVE GAME] Query failed:', error);
        return null;
      }
    },
    enabled: !!gameId,
    refetchInterval: 2000, // Poll every 2 seconds for faster sync!
  });
}

export function useGameOdds(gameId: string | null) {
  return useQuery({
    queryKey: ['odds', gameId],
    queryFn: async () => {
      const bettingClient = getBettingClient();
      if (!gameId || !bettingClient) return null;
      const data = await bettingClient.request<{ gameOdds: GameOdds }>(GET_GAME_ODDS, { gameId });
      return data.gameOdds;
    },
    enabled: !!gameId,
    refetchInterval: 10000, // Poll every 10 seconds for odds updates
  });
}

export function useBetPool(gameId: string | null) {
  return useQuery({
    queryKey: ['pool', gameId],
    queryFn: async () => {
      const bettingClient = getBettingClient();
      if (!gameId || !bettingClient) return null;
      const data = await bettingClient.request<{ betPool: BetPool }>(GET_BET_POOL, { gameId });
      return data.betPool;
    },
    enabled: !!gameId,
    refetchInterval: 5000,
  });
}

export function useMakeMove() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ gameId, from, to, promotion }: {
      gameId: string;
      from: string;
      to: string;
      promotion?: string
    }) => {
      const client = getGameClient();
      return await client.request(MAKE_MOVE, { gameId, from, to, promotion });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['game'] });
      queryClient.invalidateQueries({ queryKey: ['activeGames'] });
    },
    onError: (error) => {
      console.error('Failed to make move:', error);
    },
  });
}

export function useCreateGame() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ opponentChain, timeControl }: { opponentChain: string; timeControl: number }) => {
      const client = getGameClient();
      try {
        return await client.request(CREATE_GAME, { opponentChain, timeControl });
      } catch (error: any) {
        // Linera mutations return certificate hash in non-standard format
        // The mutation succeeds but graphql-request treats it as error
        // Check if it's actually a success (has data field with hash)
        if (error?.response?.error && typeof error.response.error === 'string') {
          const parsed = JSON.parse(error.response.error);
          if (parsed.data && typeof parsed.data === 'string') {
            // This is actually success - return the hash
            return parsed.data;
          }
        }
        throw error;
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['activeGames'] });
    },
    onError: (error) => {
      console.error('Failed to create game:', error);
    },
  });
}

export function usePlaceBet() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ gameId, outcome, amount }: {
      gameId: string;
      outcome: string;
      amount: number
    }) => {
      const bettingClient = getBettingClient();
      if (!bettingClient) throw new Error('Betting contract not deployed');
      return await bettingClient.request(PLACE_BET, { gameId, outcome, amount });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pool'] });
      queryClient.invalidateQueries({ queryKey: ['odds'] });
    },
    onError: (error) => {
      console.error('Failed to place bet:', error);
    },
  });
}
