import { GraphQLClient, gql } from 'graphql-request';
import { ChessGame, GameState, GameStatus } from '../core/types';
import { StockfishEngine } from './stockfishEngine';
import { PositionAnalyzer } from '../core/operations/analyzePosition';
import { OddsUpdater } from '../core/operations/updateOdds';
import { config } from '../config';

export class GameMonitor {
  private activeGames: Map<string, GameState> = new Map();
  private stockfish: StockfishEngine;
  private analyzer: PositionAnalyzer;
  private oddsUpdater: OddsUpdater;
  private gameClient: GraphQLClient;

  constructor(stockfish: StockfishEngine) {
    this.stockfish = stockfish;
    this.analyzer = new PositionAnalyzer(stockfish);
    this.oddsUpdater = new OddsUpdater();
    this.gameClient = new GraphQLClient(
      config.serviceUrl + '/chains/' + config.chainId + '/applications/' + config.gameAppId
    );
  }

  async fetchActiveGames(): Promise<ChessGame[]> {
    try {
      const query = gql`
        query GetActiveGames {
          activeGames {
            id
            white_player
            black_player
            status
            current_fen
            move_count
            result
            created_at
          }
        }
      `;

      const response: any = await this.gameClient.request(query);
      return response.activeGames || [];
    } catch (error) {
      console.error('Failed to fetch active games:', error);
      return [];
    }
  }

  async fetchGame(gameId: string): Promise<ChessGame | null> {
    try {
      const query = gql`
        query GetGame($gameId: String!) {
          game(gameId: $gameId) {
            id
            white_player
            black_player
            status
            current_fen
            move_count
            result
            created_at
          }
        }
      `;

      const response: any = await this.gameClient.request(query, { gameId });
      return response.game || null;
    } catch (error) {
      console.error('Failed to fetch game:', gameId, error);
      return null;
    }
  }

  async monitorGame(gameId: string): Promise<void> {
    const game = await this.fetchGame(gameId);
    
    if (!game) {
      console.warn('Game not found:', gameId);
      this.activeGames.delete(gameId);
      return;
    }

    if (game.status === GameStatus.Active) {
      const cachedState = this.activeGames.get(gameId);
      
      const shouldAnalyze = 
        !cachedState || 
        cachedState.game.move_count !== game.move_count ||
        (Date.now() - cachedState.lastChecked) > 60000;

      if (shouldAnalyze) {
        console.log('Analyzing game:', gameId, '(move', game.move_count + ')');
        
        try {
          const analysis = await this.analyzer.analyze(
            game.current_fen,
            config.stockfishDepth
          );

          const odds = {
            white_win: analysis.odds.white_win,
            black_win: analysis.odds.black_win,
            draw: analysis.odds.draw,
            evaluation: analysis.evaluation,
            last_updated: Date.now(),
          };

          await this.oddsUpdater.updateOdds(gameId, odds);

          this.activeGames.set(gameId, {
            game,
            lastAnalysis: analysis,
            lastChecked: Date.now(),
          });
        } catch (error) {
          console.error('Failed to analyze game:', gameId, error);
        }
      }
    } else if (game.status === GameStatus.Finished) {
      console.log('Game finished:', gameId, 'Result:', game.result);
      this.activeGames.delete(gameId);
    } else {
      console.log('Game not active:', gameId, 'Status:', game.status);
    }
  }

  async monitorAllGames(): Promise<void> {
    const games = await this.fetchActiveGames();
    
    console.log('Monitoring', games.length, 'active games');
    
    for (const game of games) {
      if (game.status === GameStatus.Active) {
        await this.monitorGame(game.id);
      }
    }

    const staleGames: string[] = [];
    for (const [gameId, state] of this.activeGames.entries()) {
      if (!games.find(g => g.id === gameId)) {
        staleGames.push(gameId);
      }
    }
    
    for (const gameId of staleGames) {
      console.log('Removing stale game from cache:', gameId);
      this.activeGames.delete(gameId);
    }
  }

  getActiveGameCount(): number {
    return this.activeGames.size;
  }
}
