import { StockfishEngine } from '../../workers/stockfishEngine';
import { StockfishAnalysis } from '../types';

export class PositionAnalyzer {
  constructor(private engine: StockfishEngine) {}

  async analyze(fen: string, depth: number): Promise<StockfishAnalysis> {
    try {
      const analysis = await this.engine.analyzePosition(fen, depth);
      
      const evalPawns = (analysis.evaluation / 100).toFixed(2);
      const whiteOdds = (analysis.odds.white_win / 100).toFixed(2);
      const blackOdds = (analysis.odds.black_win / 100).toFixed(2);
      const drawOdds = (analysis.odds.draw / 100).toFixed(2);
      
      console.log('Position analysis:', fen);
      console.log('  Evaluation:', analysis.evaluation, 'cp (', evalPawns, 'pawns)');
      console.log('  Best move:', analysis.bestMove || 'none');
      console.log('  Odds: W:', whiteOdds, 'x B:', blackOdds, 'x D:', drawOdds, 'x');
      
      return analysis;
    } catch (error) {
      console.error('Position analysis failed:', error);
      throw error;
    }
  }
}
