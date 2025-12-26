import { GraphQLClient } from 'graphql-request';

const GRAPHQL_URL = import.meta.env.VITE_LINERA_GRAPHQL_URL || 'http://localhost:8080';
const GAME_APP_ID = import.meta.env.VITE_GAME_APP_ID || '';
const BETTING_APP_ID = import.meta.env.VITE_BETTING_APP_ID || '';

// Helper to get chain ID from localStorage
const getUserChainId = (): string => {
  return localStorage.getItem('linera_chain_id') || import.meta.env.VITE_CHAIN_ID || '';
};

// Helper to create game client with current user chain
export const getGameClient = (): GraphQLClient => {
  const chainId = getUserChainId();
  return new GraphQLClient(
    `${GRAPHQL_URL}/chains/${chainId}/applications/${GAME_APP_ID}`
  );
};

// Helper to create betting client with current user chain
export const getBettingClient = (): GraphQLClient | null => {
  if (!BETTING_APP_ID) return null;
  const chainId = getUserChainId();
  return new GraphQLClient(
    `${GRAPHQL_URL}/chains/${chainId}/applications/${BETTING_APP_ID}`
  );
};

// Legacy exports for backward compatibility
export const gameClient = getGameClient();
export const bettingClient = getBettingClient();

// Game Queries
export const GET_ACTIVE_GAMES = `
  query GetActiveGames {
    allGames {
      id
      whitePlayer
      blackPlayer
      status
      currentFen
      moveCount
      result
      createdAt
      updatedAt
    }
  }
`;

export const GET_GAME = `
  query GetGame($gameId: String!) {
    game(gameId: $gameId) {
      id
      whitePlayer
      blackPlayer
      status
      currentFen
      moveCount
      result
      createdAt
      updatedAt
    }
  }
`;

// Betting Queries
export const GET_GAME_ODDS = `
  query GetGameOdds($gameId: String!) {
    gameOdds(gameId: $gameId) {
      whiteOdds
      blackOdds
      drawOdds
    }
  }
`;

export const GET_BET_POOL = `
  query GetBetPool($gameId: String!) {
    betPool(gameId: $gameId) {
      gameId
      totalPool
      whitePool
      blackPool
      drawPool
      isLocked
    }
  }
`;

export const GET_USER_BETS = `
  query GetUserBets($address: String!) {
    userBets(address: $address) {
      id
      gameId
      bettor
      amount
      betOn
      oddsAtPlacement
      status
      placedAt
    }
  }
`;

// Mutations
export const MAKE_MOVE = `
  mutation PlaceMove($gameId: String!, $from: String!, $to: String!, $promotion: String) {
    placeMove(gameId: $gameId, from: $from, to: $to, promotion: $promotion)
  }
`;

export const CREATE_GAME = `
  mutation CreateGame($opponentChain: String!, $timeControl: Int!) {
    createGame(opponentChain: $opponentChain, timeControl: $timeControl)
  }
`;

export const PLACE_BET = `
  mutation PlaceBet($gameId: String!, $outcome: String!, $amount: Int!) {
    placeBet(gameId: $gameId, outcome: $outcome, amount: $amount)
  }
`;

export const PROCESS_INBOX = `
  mutation ProcessInbox {
    processInbox
  }
`;
