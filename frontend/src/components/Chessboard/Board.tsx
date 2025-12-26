import { useState, useEffect } from 'react';
import { Chessboard } from 'react-chessboard';
import { Chess } from 'chess.js';
import { useGame } from '@/hooks/useGames';

interface ChessBoardProps {
  gameId: string;
}

export function ChessBoard({ gameId }: ChessBoardProps) {
  const [game, setGame] = useState(new Chess());

  const { data: currentGame } = useGame(gameId);

  useEffect(() => {
    if (currentGame?.current_fen) {
      const newGame = new Chess(currentGame.current_fen);
      setGame(newGame);
    }
  }, [currentGame]);

  const moveHistory: any[] = [];

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      {/* Game Info */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold text-white">
            Game {gameId.slice(5, 11)}
          </h2>
          <span className={`px-3 py-1 rounded-full text-sm font-semibold ${
            currentGame?.status === 'Active' ? 'bg-green-500/20 text-green-400' :
            currentGame?.status === 'Finished' ? 'bg-blue-500/20 text-blue-400' :
            'bg-yellow-500/20 text-yellow-400'
          }`}>
            {currentGame?.status || 'Loading...'}
          </span>
        </div>

        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="bg-gray-700 p-3 rounded">
            <div className="text-gray-400 mb-1">White</div>
            <div className="font-mono text-xs text-white truncate">
              {currentGame?.white_player.slice(0, 20) || '...'}
            </div>
          </div>
          <div className="bg-gray-700 p-3 rounded">
            <div className="text-gray-400 mb-1">Black</div>
            <div className="font-mono text-xs text-white truncate">
              {currentGame?.black_player.slice(0, 20) || '...'}
            </div>
          </div>
        </div>
      </div>

      {/* Chessboard */}
      <div className="mb-4">
        <Chessboard
          position={game.fen()}
          boardWidth={400}
          customBoardStyle={{
            borderRadius: '8px',
            boxShadow: '0 4px 6px rgba(0, 0, 0, 0.3)',
          }}
        />
      </div>

      {/* Move History */}
      <div className="bg-gray-700 rounded-lg p-4">
        <h3 className="text-sm font-semibold text-white mb-3">
          Move History ({moveHistory.length} moves)
        </h3>
        <div className="max-h-32 overflow-y-auto">
          {moveHistory.length === 0 ? (
            <div className="text-gray-400 text-sm text-center py-4">
              No moves yet
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-2 text-sm">
              {moveHistory.map((move: any, index: number) => (
                <div key={index} className="flex items-center space-x-2 text-gray-300">
                  <span className="text-gray-500">{Math.floor(index / 2) + 1}.</span>
                  <span className="font-semibold">{move.san}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Result */}
      {currentGame?.result && (
        <div className="mt-4 bg-blue-500/20 border border-blue-500/30 rounded-lg p-4 text-center">
          <div className="text-blue-400 font-semibold">
            {currentGame.result === 'WhiteWins' ? 'White Wins!' :
             currentGame.result === 'BlackWins' ? 'Black Wins!' :
             'Draw!'}
          </div>
        </div>
      )}
    </div>
  );
}
