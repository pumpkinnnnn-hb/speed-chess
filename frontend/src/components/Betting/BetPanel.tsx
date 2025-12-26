import { useState } from 'react';
import { usePlaceBet, useGameOdds } from '@/hooks/useGames';

interface BetPanelProps {
  gameId: string;
}

type BetSelection = 'WhiteWins' | 'BlackWins' | 'Draw';

export function BetPanel({ gameId }: BetPanelProps) {
  const [amount, setAmount] = useState('100');
  const [selection, setSelection] = useState<BetSelection>('WhiteWins');
  const placeBet = usePlaceBet();

  const { data: odds } = useGameOdds(gameId);

  const handlePlaceBet = async () => {
    try {
      await placeBet.mutateAsync({
        gameId,
        outcome: selection,
        amount: parseInt(amount)
      });
    } catch (error) {
      console.error('Failed to place bet:', error);
    }
  };

  const formatOdds = (basisPoints: number) => {
    return (basisPoints / 100).toFixed(2) + 'x';
  };

  const formatEval = (centipawns: number) => {
    const pawns = (centipawns / 100).toFixed(2);
    return centipawns > 0 ? `+${pawns}` : pawns;
  };

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <h2 className="text-xl font-semibold text-white mb-4">Place Your Bet</h2>

      {/* Stockfish Evaluation */}
      {odds && (
        <div className="bg-gray-700 rounded-lg p-4 mb-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-400">Stockfish Evaluation</span>
            <span className="text-sm text-gray-500">
              {new Date(odds.last_updated / 1000).toLocaleTimeString()}
            </span>
          </div>
          <div className="text-2xl font-bold text-white">
            {formatEval(odds.evaluation)} pawns
          </div>
          <div className="mt-2 h-2 bg-gray-600 rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-white to-blue-500 transition-all duration-500"
              style={{
                width: `${Math.min(100, Math.max(0, 50 + odds.evaluation / 10))}%`,
              }}
            />
          </div>
          <div className="flex justify-between text-xs text-gray-400 mt-1">
            <span>White</span>
            <span>Black</span>
          </div>
        </div>
      )}

      {/* Betting Options */}
      <div className="grid grid-cols-3 gap-3 mb-4">
        <button
          onClick={() => setSelection('WhiteWins')}
          className={`p-4 rounded-lg border-2 transition-all ${
            selection === 'WhiteWins'
              ? 'border-blue-500 bg-blue-500/20'
              : 'border-gray-600 bg-gray-700 hover:border-gray-500'
          }`}
        >
          <div className="text-white font-semibold mb-1">White</div>
          <div className="text-2xl font-bold text-blue-400">
            {odds ? formatOdds(odds.white_win) : '...'}
          </div>
        </button>

        <button
          onClick={() => setSelection('Draw')}
          className={`p-4 rounded-lg border-2 transition-all ${
            selection === 'Draw'
              ? 'border-yellow-500 bg-yellow-500/20'
              : 'border-gray-600 bg-gray-700 hover:border-gray-500'
          }`}
        >
          <div className="text-white font-semibold mb-1">Draw</div>
          <div className="text-2xl font-bold text-yellow-400">
            {odds ? formatOdds(odds.draw) : '...'}
          </div>
        </button>

        <button
          onClick={() => setSelection('BlackWins')}
          className={`p-4 rounded-lg border-2 transition-all ${
            selection === 'BlackWins'
              ? 'border-purple-500 bg-purple-500/20'
              : 'border-gray-600 bg-gray-700 hover:border-gray-500'
          }`}
        >
          <div className="text-white font-semibold mb-1">Black</div>
          <div className="text-2xl font-bold text-purple-400">
            {odds ? formatOdds(odds.black_win) : '...'}
          </div>
        </button>
      </div>

      {/* Bet Amount */}
      <div className="mb-4">
        <label className="block text-sm font-semibold text-white mb-2">
          Bet Amount
        </label>
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="Enter amount..."
        />
        <div className="flex gap-2 mt-2">
          {['50', '100', '500', '1000'].map((preset) => (
            <button
              key={preset}
              onClick={() => setAmount(preset)}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-gray-300 py-2 rounded text-sm"
            >
              {preset}
            </button>
          ))}
        </div>
      </div>

      {/* Place Bet Button */}
      <button
        onClick={handlePlaceBet}
        className="w-full bg-green-600 hover:bg-green-700 text-white font-semibold py-4 rounded-lg transition-colors"
      >
        Place Bet
      </button>

      {/* Potential Winnings */}
      {odds && (
        <div className="mt-4 bg-gray-700 rounded-lg p-4">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Potential Winnings:</span>
            <span className="text-green-400 font-bold">
              {selection === 'WhiteWins' ? (parseInt(amount) * odds.white_win / 10000).toFixed(0) :
               selection === 'BlackWins' ? (parseInt(amount) * odds.black_win / 10000).toFixed(0) :
               (parseInt(amount) * odds.draw / 10000).toFixed(0)} tokens
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
