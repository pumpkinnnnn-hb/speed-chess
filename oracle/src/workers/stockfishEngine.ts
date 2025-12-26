import { spawn, ChildProcess } from 'child_process';
import { StockfishAnalysis } from '../core/types';

export class StockfishEngine {
  private process: ChildProcess | null = null;
  private ready: boolean = false;
  private outputBuffer: string = '';

  async start(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        // Try to spawn stockfish binary
        this.process = spawn('stockfish');

        this.process.on('error', (err) => {
          console.error('❌ Failed to start Stockfish:', err.message);
          reject(new Error('Stockfish binary not found. Please install stockfish.'));
        });

        this.process.stdout?.on('data', (data) => {
          const output = data.toString();
          this.outputBuffer += output;

          if (output.includes('uciok')) {
            console.log('✅ Stockfish engine initialized');
            this.ready = true;
            resolve();
          }
        });

        this.process.stderr?.on('data', (data) => {
          console.error('Stockfish stderr:', data.toString());
        });

        // Initialize UCI protocol
        this.process.stdin?.write('uci\n');
        this.process.stdin?.write('setoption name Threads value 4\n');
        this.process.stdin?.write('setoption name Hash value 256\n');
        this.process.stdin?.write('isready\n');

        // Timeout after 5 seconds
        setTimeout(() => {
          if (!this.ready) {
            reject(new Error('Stockfish initialization timeout'));
          }
        }, 5000);
      } catch (error) {
        reject(error);
      }
    });
  }

  async analyzePosition(fen: string, depth: number = 15): Promise<StockfishAnalysis> {
    if (!this.ready || !this.process) {
      throw new Error('Stockfish engine not ready');
    }

    return new Promise((resolve, reject) => {
      let bestMove: string | null = null;
      let evaluation: number = 0;
      let currentDepth: number = 0;

      const timeout = setTimeout(() => {
        reject(new Error('Stockfish analysis timeout'));
      }, 10000); // 10 second timeout

      const dataHandler = (data: Buffer) => {
        const output = data.toString();
        const lines = output.split('\n');

        for (const line of lines) {
          // Parse depth
          const depthMatch = line.match(/depth (\d+)/);
          if (depthMatch) {
            currentDepth = parseInt(depthMatch[1]);
          }

          // Parse evaluation (centipawns)
          const cpMatch = line.match(/score cp (-?\d+)/);
          if (cpMatch) {
            evaluation = parseInt(cpMatch[1]);
          }

          // Parse mate score
          const mateMatch = line.match(/score mate (-?\d+)/);
          if (mateMatch) {
            const mateIn = parseInt(mateMatch[1]);
            // Convert mate to large centipawn value
            evaluation = mateIn > 0 ? 10000 : -10000;
          }

          // Parse best move
          const moveMatch = line.match(/bestmove ([a-h][1-8][a-h][1-8][qrbn]?)/);
          if (moveMatch) {
            bestMove = moveMatch[1];
            
            // Analysis complete
            clearTimeout(timeout);
            this.process!.stdout?.off('data', dataHandler);

            const odds = this.calculateOdds(evaluation);
            
            resolve({
              fen,
              evaluation,
              bestMove,
              depth: currentDepth,
              odds,
            });
            return;
          }
        }
      };

      this.process!.stdout?.on('data', dataHandler);

      // Send analysis command
      this.process!.stdin?.write(`position fen ${fen}\n`);
      this.process!.stdin?.write(`go depth ${depth}\n`);
    });
  }

  private calculateOdds(centipawns: number): {
    white_win: number;
    black_win: number;
    draw: number;
  } {
    // Convert centipawns to win probability using logistic function
    // Formula: P(win) = 1 / (1 + 10^(-eval/400))
    // eval is in pawns (centipawns / 100)
    
    const evalPawns = centipawns / 100.0;
    
    // White win probability (logistic curve)
    const whiteWinProb = 1.0 / (1.0 + Math.pow(10, -evalPawns / 4.0));
    
    // Draw probability (decreases with absolute eval)
    // Base 25% draw rate, decreasing as position becomes more decisive
    const drawProb = 0.25 * Math.exp(-Math.abs(evalPawns) / 5.0);
    
    // Black win probability (remainder)
    let blackWinProb = 1.0 - whiteWinProb - drawProb;
    
    // Normalize to ensure probabilities sum to 1.0
    const total = whiteWinProb + blackWinProb + drawProb;
    const whiteNorm = whiteWinProb / total;
    const blackNorm = blackWinProb / total;
    const drawNorm = drawProb / total;
    
    // Convert probabilities to odds (basis points)
    // Odds = 1 / probability * 10000
    // Clamp between 1.0x (10000) and 10.0x (100000)
    const whiteOdds = this.probabilityToOdds(whiteNorm);
    const blackOdds = this.probabilityToOdds(blackNorm);
    const drawOdds = this.probabilityToOdds(drawNorm);
    
    return {
      white_win: whiteOdds,
      black_win: blackOdds,
      draw: drawOdds,
    };
  }

  private probabilityToOdds(probability: number): number {
    if (probability < 0.01) {
      return 100000; // Cap at 10.0x
    }
    
    const odds = Math.round((1.0 / probability) * 10000);
    
    // Clamp between 1.0x and 10.0x
    return Math.max(10000, Math.min(100000, odds));
  }

  async stop(): Promise<void> {
    if (this.process) {
      this.process.stdin?.write('quit\n');
      this.process.kill();
      this.process = null;
      this.ready = false;
      console.log('🛑 Stockfish engine stopped');
    }
  }
}
