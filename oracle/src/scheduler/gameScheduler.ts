import cron from 'node-cron';
import { GameMonitor } from '../workers/gameMonitor';

export class GameScheduler {
  private task: cron.ScheduledTask | null = null;
  private isRunning: boolean = false;

  constructor(private monitor: GameMonitor) {}

  start(): void {
    if (this.task) {
      console.log('Scheduler already running');
      return;
    }

    const intervalSeconds = Math.floor(30);
    const cronExpression = '*/' + intervalSeconds + ' * * * * *';

    this.task = cron.schedule(cronExpression, async () => {
      if (this.isRunning) {
        console.log('Previous monitoring cycle still running, skipping...');
        return;
      }

      this.isRunning = true;
      
      try {
        const timestamp = new Date().toISOString();
        console.log('');
        console.log('=== Game Monitoring Cycle ===', timestamp);
        
        await this.monitor.monitorAllGames();
        
        const activeCount = this.monitor.getActiveGameCount();
        console.log('Active games in cache:', activeCount);
        console.log('=============================');
      } catch (error) {
        console.error('Monitoring cycle error:', error);
      } finally {
        this.isRunning = false;
      }
    });

    console.log('Game monitoring scheduler started');
    console.log('Polling every 30 seconds');
  }

  stop(): void {
    if (this.task) {
      this.task.stop();
      this.task = null;
      console.log('Scheduler stopped');
    }
  }

  async runOnce(): Promise<void> {
    console.log('Running single monitoring cycle...');
    await this.monitor.monitorAllGames();
    console.log('Monitoring cycle complete');
  }
}
