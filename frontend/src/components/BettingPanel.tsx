import { useState } from 'react';
import { useGameStore } from '@/store/gameStore';
import { useGameOdds, useBetPool, usePlaceBet } from '@/hooks/useGames';

const QUICK_AMOUNTS = [10, 50, 100];

export function BettingPanel() {
  const { activeGameId, wallet, betAmount, setBetAmount, selectedBetOutcome, setSelectedBetOutcome } = useGameStore();
  const { data: odds } = useGameOdds(activeGameId);
  const { data: pool } = useBetPool(activeGameId);
  const placeBet = usePlaceBet();
  const [customAmount, setCustomAmount] = useState('');

  const formatOdds = (basisPoints: number) => {
    return (basisPoints / 10000).toFixed(2);
  };

  const handleQuickAmount = (amount: number) => {
    setBetAmount(amount.toString());
    setCustomAmount('');
  };

  const handleMaxAmount = () => {
    setBetAmount(wallet.balance.toString());
    setCustomAmount(wallet.balance.toString());
  };

  const handlePlaceBet = async () => {
    if (!activeGameId || !selectedBetOutcome || !betAmount) return;

    const amount = parseInt(betAmount);
    if (isNaN(amount) || amount <= 0 || amount > wallet.balance) return;

    try {
      await placeBet.mutateAsync({
        gameId: activeGameId,
        outcome: selectedBetOutcome,
        amount,
      });

      setBetAmount('');
      setCustomAmount('');
      setSelectedBetOutcome(null);
    } catch (error) {
      console.error('Failed to place bet:', error);
    }
  };

  if (!activeGameId) {
    return (
      <div className="border border-terminal-border bg-terminal-surface p-6 text-center">
        <div className="text-terminal-muted text-sm">
          Select a game to start betting
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="border border-terminal-border neon-border bg-terminal-surface p-4">
        <h3 className="text-sm font-bold mb-4">Live Odds</h3>

        <div className="space-y-2">
          <button
            onClick={() => setSelectedBetOutcome('White')}
            className={
              selectedBetOutcome === 'White'
                ? 'w-full terminal-button-primary text-left'
                : 'w-full terminal-button text-left'
            }
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span>♔</span>
                <span>White Wins</span>
              </div>
              <span className="neon-text font-bold">
                {odds ? formatOdds(odds.whiteOdds) + 'x' : '...'}
              </span>
            </div>
          </button>

          <button
            onClick={() => setSelectedBetOutcome('Black')}
            className={
              selectedBetOutcome === 'Black'
                ? 'w-full terminal-button-primary text-left'
                : 'w-full terminal-button text-left'
            }
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span>♚</span>
                <span>Black Wins</span>
              </div>
              <span className="neon-text font-bold">
                {odds ? formatOdds(odds.blackOdds) + 'x' : '...'}
              </span>
            </div>
          </button>

          <button
            onClick={() => setSelectedBetOutcome('Draw')}
            className={
              selectedBetOutcome === 'Draw'
                ? 'w-full terminal-button-primary text-left'
                : 'w-full terminal-button text-left'
            }
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span>=</span>
                <span>Draw</span>
              </div>
              <span className="neon-text font-bold">
                {odds ? formatOdds(odds.drawOdds) + 'x' : '...'}
              </span>
            </div>
          </button>
        </div>
      </div>

      {selectedBetOutcome && (
        <div className="border border-terminal-border bg-terminal-surface p-4">
          <h3 className="text-sm font-bold mb-4">Bet Amount</h3>

          <div className="grid grid-cols-4 gap-2 mb-3">
            {QUICK_AMOUNTS.map((amount) => (
              <button
                key={amount}
                onClick={() => handleQuickAmount(amount)}
                className={
                  betAmount === amount.toString()
                    ? 'terminal-button-primary'
                    : 'terminal-button'
                }
              >
                {amount}
              </button>
            ))}
            <button
              onClick={handleMaxAmount}
              className={
                betAmount === wallet.balance.toString()
                  ? 'terminal-button-primary'
                  : 'terminal-button'
              }
            >
              MAX
            </button>
          </div>

          <input
            type="number"
            value={customAmount}
            onChange={(e) => {
              setCustomAmount(e.target.value);
              setBetAmount(e.target.value);
            }}
            placeholder="Custom amount"
            className="terminal-input mb-4"
          />

          {betAmount && (
            <div className="space-y-2 text-sm mb-4">
              <div className="flex justify-between text-terminal-muted">
                <span>Potential Payout</span>
                <span className="neon-text font-bold">
                  {odds && (parseInt(betAmount) * parseFloat(formatOdds(
                    selectedBetOutcome === 'White' ? odds.whiteOdds :
                    selectedBetOutcome === 'Black' ? odds.blackOdds :
                    odds.drawOdds
                  ))).toFixed(2)}
                </span>
              </div>
            </div>
          )}

          <button
            onClick={handlePlaceBet}
            disabled={!betAmount || placeBet.isPending}
            className="w-full terminal-button-primary disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {placeBet.isPending ? 'Placing Bet...' : 'Place Bet'}
          </button>
        </div>
      )}

      {pool && (
        <div className="border border-terminal-border bg-terminal-surface p-4">
          <h3 className="text-sm font-bold mb-4">Bet Pool</h3>

          <div className="space-y-3">
            <div>
              <div className="flex justify-between text-xs text-terminal-muted mb-1">
                <span>♔ White Pool</span>
                <span>{(pool.whitePool / 1000000).toFixed(2)}</span>
              </div>
              <div className="h-1.5 bg-terminal-bg border border-terminal-border overflow-hidden">
                <div
                  className="h-full bg-terminal-neon"
                  style={{
                    width: pool.totalPool ? (pool.whitePool / pool.totalPool * 100) + '%' : '0%'
                  }}
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-xs text-terminal-muted mb-1">
                <span>♚ Black Pool</span>
                <span>{(pool.blackPool / 1000000).toFixed(2)}</span>
              </div>
              <div className="h-1.5 bg-terminal-bg border border-terminal-border overflow-hidden">
                <div
                  className="h-full bg-terminal-neon"
                  style={{
                    width: pool.totalPool ? (pool.blackPool / pool.totalPool * 100) + '%' : '0%'
                  }}
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-xs text-terminal-muted mb-1">
                <span>= Draw Pool</span>
                <span>{(pool.drawPool / 1000000).toFixed(2)}</span>
              </div>
              <div className="h-1.5 bg-terminal-bg border border-terminal-border overflow-hidden">
                <div
                  className="h-full bg-terminal-neon"
                  style={{
                    width: pool.totalPool ? (pool.drawPool / pool.totalPool * 100) + '%' : '0%'
                  }}
                />
              </div>
            </div>

            <div className="pt-2 border-t border-terminal-border">
              <div className="flex justify-between text-sm">
                <span className="text-terminal-muted">Total Pool</span>
                <span className="neon-text font-bold">
                  {(pool.totalPool / 1000000).toFixed(2)}
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
