import { OracleConfig } from './core/types';

export const config: OracleConfig = {
  // These will be populated by run.bash deployment script
  gameAppId: process.env.GAME_APP_ID || '',
  bettingAppId: process.env.BETTING_APP_ID || '',
  serviceUrl: process.env.LINERA_SERVICE_URL || 'http://localhost:9001',
  chainId: process.env.ORACLE_CHAIN_ID || '',
  
  // Oracle configuration
  pollingInterval: 30000, // Check games every 30 seconds
  stockfishDepth: 15, // Depth 15 for faster analysis (~2s)
};

export function validateConfig(): void {
  if (!config.gameAppId) {
    throw new Error('GAME_APP_ID environment variable not set');
  }
  if (!config.bettingAppId) {
    throw new Error('BETTING_APP_ID environment variable not set');
  }
  if (!config.chainId) {
    throw new Error('ORACLE_CHAIN_ID environment variable not set');
  }
  console.log('✅ Configuration validated');
  console.log(`   Game App: ${config.gameAppId}`);
  console.log(`   Betting App: ${config.bettingAppId}`);
  console.log(`   Service URL: ${config.serviceUrl}`);
}
