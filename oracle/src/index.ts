import { StockfishEngine } from './workers/stockfishEngine';
import { GameMonitor } from './workers/gameMonitor';
import { GameScheduler } from './scheduler/gameScheduler';
import { config, validateConfig } from './config';

async function main() {
  console.log('========================================');
  console.log('  Speed Chess Betting Oracle');
  console.log('  Stockfish Position Analysis Service');
  console.log('========================================');
  console.log('');

  try {
    validateConfig();
    console.log('');

    console.log('Starting Stockfish engine...');
    const stockfish = new StockfishEngine();
    await stockfish.start();
    console.log('');

    console.log('Initializing game monitor...');
    const gameMonitor = new GameMonitor(stockfish);
    console.log('Game monitor initialized');
    console.log('');

    console.log('Starting scheduler...');
    const scheduler = new GameScheduler(gameMonitor);
    
    await scheduler.runOnce();
    console.log('');
    
    scheduler.start();
    console.log('');
    console.log('Oracle service running...');
    console.log('Press Ctrl+C to stop');
    console.log('');

    process.on('SIGINT', async () => {
      console.log('');
      console.log('Shutting down gracefully...');
      scheduler.stop();
      await stockfish.stop();
      console.log('Oracle service stopped');
      process.exit(0);
    });

    process.on('SIGTERM', async () => {
      console.log('');
      console.log('Shutting down gracefully...');
      scheduler.stop();
      await stockfish.stop();
      console.log('Oracle service stopped');
      process.exit(0);
    });

  } catch (error) {
    console.error('Fatal error:', error);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error('Unhandled error:', error);
  process.exit(1);
});
