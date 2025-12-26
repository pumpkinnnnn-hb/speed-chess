import { useGameStore } from '@/store/gameStore';
import { useActiveGames, useCreateGame } from '@/hooks/useGames';
import { useWallet } from '@/hooks/useWallet';
import { GameStatus } from '@/types';

export function GamesList() {
  const { data: games, isLoading, error } = useActiveGames();
  const { activeGameId, setActiveGame } = useGameStore();
  const createGame = useCreateGame();
  const wallet = useWallet();

  console.log('🎯 GamesList render - isLoading:', isLoading, 'games:', games?.length || 0, 'error:', error);

  if (isLoading || wallet.isLoading) {
    return (
      <div className="border border-terminal-border bg-terminal-surface p-4">
        <div className="text-terminal-muted text-sm loading-dots">
          {wallet.isLoading ? 'Initializing wallet...' : 'Loading games'}
        </div>
      </div>
    );
  }

  if (wallet.error) {
    return (
      <div className="border border-terminal-border bg-terminal-surface p-4 text-center">
        <div className="text-red-500 text-sm mb-3">Wallet Error: {wallet.error}</div>
        <button onClick={wallet.resetWallet} className="terminal-button-primary">
          Retry
        </button>
      </div>
    );
  }

  const handleCreateGame = async () => {
    console.log('🎮 Create game button clicked');
    console.log('🎮 Wallet chain ID:', wallet.chainId);

    if (!wallet.chainId) {
      console.error('❌ No wallet chain available');
      alert('Wallet not initialized. Please refresh the page.');
      return;
    }

    try {
      const opponentChain = import.meta.env.VITE_PLAYER2_CHAIN || wallet.chainId;

      console.log('🎮 Creating game with params:', {
        opponentChain: opponentChain,
        timeControl: 300
      });

      const result = await createGame.mutateAsync({
        opponentChain: opponentChain,
        timeControl: 300 // 5 minutes default
      });

      console.log('✅ Game created successfully! Certificate:', result);
    } catch (error) {
      console.error('❌ Failed to create game:', error);
      alert(`Failed to create game: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  if (!games || games.length === 0) {
    return (
      <div className="border border-terminal-border bg-terminal-surface p-4 text-center">
        <div className="text-terminal-muted text-sm mb-3">No games found (v2.0 - Debug Mode)</div>
        <button
          onClick={handleCreateGame}
          disabled={createGame.isPending}
          className="terminal-button-primary"
        >
          {createGame.isPending ? 'Creating...' : '🎮 Create New Game (DEBUG)'}
        </button>
      </div>
    );
  }

  return (
    <div className="border border-terminal-border bg-terminal-surface">
      <div className="border-b border-terminal-border p-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold">Active Games</h3>
          <button
            onClick={handleCreateGame}
            disabled={createGame.isPending}
            className="terminal-button-primary text-xs px-3 py-1"
          >
            {createGame.isPending ? 'Creating...' : '+ New Game'}
          </button>
        </div>
      </div>

      <div className="divide-y divide-terminal-border max-h-[600px] overflow-y-auto">
        {games.map((game) => (
          <button
            key={game.id}
            onClick={() => setActiveGame(game.id)}
            className={
              activeGameId === game.id
                ? 'w-full p-3 text-left bg-terminal-neon/10 border-l-2 border-terminal-neon hover:bg-terminal-neon/15 transition-colors'
                : 'w-full p-3 text-left hover:bg-terminal-surface/50 transition-colors'
            }
          >
            <div className="flex items-center justify-between mb-2">
              <div className="text-xs font-mono text-terminal-muted">
                {game.id.slice(0, 12)}...
              </div>
              <div className={
                game.status === GameStatus.Active
                  ? 'text-xs text-terminal-neon'
                  : 'text-xs text-terminal-muted'
              }>
                {game.status}
              </div>
            </div>

            <div className="flex items-center gap-2 text-sm mb-1">
              <span>♔</span>
              <span className="font-mono text-xs text-terminal-muted">
                {game.whitePlayer.slice(0, 10)}...
              </span>
            </div>

            <div className="flex items-center gap-2 text-sm mb-2">
              <span>♚</span>
              <span className="font-mono text-xs text-terminal-muted">
                {game.blackPlayer.slice(0, 10)}...
              </span>
            </div>

            <div className="flex items-center justify-between text-xs text-terminal-muted">
              <span>Moves: {game.moveCount}</span>
              <span>{new Date(game.createdAt).toLocaleTimeString()}</span>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
