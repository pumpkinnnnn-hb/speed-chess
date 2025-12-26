import { useActiveGames } from '@/hooks/useGames';

import { ChessGame } from '@/types';

interface GameListProps {
  onSelectGame: (gameId: string) => void;
  selectedGameId: string | null;
}

export function GameList({ onSelectGame, selectedGameId }: GameListProps) {
  const { data, isLoading: loading, error } = useActiveGames();

  if (loading) {
    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold text-white mb-4">Active Games</h2>
        <div className="text-gray-400 text-center py-8">Loading games...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold text-white mb-4">Active Games</h2>
        <div className="text-red-400 text-center py-8">Error loading games</div>
      </div>
    );
  }

  const games = data || [];

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <h2 className="text-xl font-semibold text-white mb-4 flex items-center justify-between">
        <span>Active Games</span>
        <span className="text-sm text-gray-400 font-normal">{games.length} live</span>
      </h2>

      <div className="space-y-3">
        {games.length === 0 ? (
          <div className="text-gray-400 text-center py-8">
            <p>No active games</p>
            <p className="text-sm mt-2">Create a new game to get started</p>
          </div>
        ) : (
          games.map((game: ChessGame) => (
            <button
              key={game.id}
              onClick={() => onSelectGame(game.id)}
              className={`w-full text-left p-4 rounded-lg transition-all ${
                selectedGameId === game.id
                  ? 'bg-blue-600 ring-2 ring-blue-400'
                  : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              <div className="flex items-center justify-between mb-2">
                <span className="font-semibold text-white text-sm">Game {game.id.slice(5, 11)}</span>
                <span className="text-xs bg-green-500/20 text-green-400 px-2 py-1 rounded">
                  {game.status}
                </span>
              </div>

              <div className="text-sm text-gray-300 space-y-1">
                <div className="flex items-center">
                  <span className="w-16">White:</span>
                  <span className="font-mono text-xs truncate">{game.whitePlayer.slice(0, 12)}...</span>
                </div>
                <div className="flex items-center">
                  <span className="w-16">Black:</span>
                  <span className="font-mono text-xs truncate">{game.blackPlayer.slice(0, 12)}...</span>
                </div>
              </div>

              <div className="mt-2 text-xs text-gray-400">
                {game.moveCount} moves played
              </div>
            </button>
          ))
        )}
      </div>

      <button className="w-full mt-4 bg-green-600 hover:bg-green-700 text-white font-semibold py-3 rounded-lg transition-colors">
        + Create New Game
      </button>
    </div>
  );
}
