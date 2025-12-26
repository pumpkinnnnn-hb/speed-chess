import { useState, useEffect } from 'react';
import { Chessboard } from 'react-chessboard';
import { Chess } from 'chess.js';
import { useGameStore } from '@/store/gameStore';
import { useMakeMove, useGame } from '@/hooks/useGames';
import { useWallet } from '@/hooks/useWallet';

export function ChessBoard() {
  const { activeGameId } = useGameStore();
  const { chainId } = useWallet();

  // CRITICAL FIX: Use useGame() hook for real-time polling with processInbox!
  const { data: activeGame } = useGame(activeGameId);

  const makeMove = useMakeMove();
  const [game, setGame] = useState(new Chess());
  
  useEffect(() => {
    console.log('♟️ [CHESSBOARD] FEN update triggered');
    console.log('  activeGame:', activeGame?.id);
    console.log('  currentFen:', activeGame?.currentFen);
    console.log('  moveCount:', activeGame?.moveCount);

    if (activeGame?.currentFen) {
      console.log('✅ [CHESSBOARD] Setting board to FEN:', activeGame.currentFen.slice(0, 50));
      const newGame = new Chess(activeGame.currentFen);
      setGame(newGame);
    } else {
      console.warn('⚠️ [CHESSBOARD] No FEN available, using starting position');
    }
  }, [activeGame?.currentFen]);

  // Determine player's color based on wallet chainId
  const playerColor = !activeGame || !chainId
    ? 'spectator'
    : activeGame.whitePlayer === chainId
      ? 'white'
      : activeGame.blackPlayer === chainId
        ? 'black'
        : 'spectator';

  console.log('🎨 [CHESSBOARD] Player color detection:');
  console.log('  Wallet chainId:', chainId);
  console.log('  Game whitePlayer:', activeGame?.whitePlayer);
  console.log('  Game blackPlayer:', activeGame?.blackPlayer);
  console.log('  Comparison result:', playerColor);
  console.log('  White match?', activeGame?.whitePlayer === chainId);
  console.log('  Black match?', activeGame?.blackPlayer === chainId);

  // Determine whose turn it is from the chess game state
  const currentTurn = game.turn(); // 'w' or 'b'
  const isMyTurn = (currentTurn === 'w' && playerColor === 'white') ||
                   (currentTurn === 'b' && playerColor === 'black');

  const onDrop = (sourceSquare: string, targetSquare: string) => {
    if (!activeGame) return false;

    // CRITICAL: Spectators cannot make moves
    if (playerColor === 'spectator') {
      console.log('❌ You are a spectator and cannot make moves!');
      return false;
    }

    // CRITICAL: Validate it's the player's turn before allowing move
    if (!isMyTurn) {
      console.log('❌ Not your turn! Current turn:', currentTurn === 'w' ? 'White' : 'Black', '| You are:', playerColor);
      return false;
    }
    
    try {
      const move = game.move({
        from: sourceSquare,
        to: targetSquare,
        promotion: 'q',
      });
      
      if (move) {
        makeMove.mutate({
          gameId: activeGame.id,
          from: sourceSquare,
          to: targetSquare,
          promotion: move.promotion ? 'q' : undefined,
        });
        
        return true;
      }
      
      return false;
    } catch (error) {
      return false;
    }
  };
  
  const evaluation = 0;
  const evalBarWidth = Math.min(Math.max((evaluation / 500) * 50 + 50, 0), 100);
  
  return (
    <div className="space-y-4">
      <div className="border border-terminal-border neon-border bg-terminal-surface p-4">
        {/* TURN & PLAYER COLOR INDICATORS */}
        <div className="mb-3 p-3 border border-terminal-border bg-terminal-bg">
          <div className="flex justify-between items-center">
            <div>
              <span className="text-terminal-muted text-sm">You are: </span>
              <span className="text-terminal-neon font-bold text-lg">
                {playerColor === 'white' ? 'WHITE' :
                 playerColor === 'black' ? 'BLACK' :
                 'SPECTATOR'}
              </span>
            </div>
            <div>
              <span className="text-terminal-muted text-sm">Current Turn: </span>
              <span className={
                isMyTurn
                  ? 'text-terminal-neon font-bold text-lg animate-pulse'
                  : 'text-terminal-muted text-lg'
              }>
                {currentTurn === 'w' ? 'WHITE' : 'BLACK'}
                {isMyTurn && ' (YOUR TURN!)'}
              </span>
            </div>
          </div>
        </div>

        <div className="relative">
          <Chessboard
            position={game.fen()}
            onPieceDrop={onDrop}
            boardOrientation={playerColor === 'black' ? 'black' : 'white'}
            boardWidth={560}
            customBoardStyle={{
              borderRadius: '0px',
              boxShadow: 'none',
            }}
            customDarkSquareStyle={{
              backgroundColor: '#0d1117',
            }}
            customLightSquareStyle={{
              backgroundColor: 'rgba(230, 237, 243, 0.1)',
            }}
            customPremoveDarkSquareStyle={{
              backgroundColor: 'rgba(0, 255, 159, 0.2)',
            }}
            customPremoveLightSquareStyle={{
              backgroundColor: 'rgba(0, 255, 159, 0.3)',
            }}
          />
        </div>
        
        <div className="mt-4 space-y-2">
          <div className="flex items-center justify-between text-xs text-terminal-muted">
            <span>Position Evaluation</span>
            <span className={evaluation > 0 ? 'text-terminal-neon' : evaluation < 0 ? 'text-terminal-error' : ''}>
              {evaluation > 0 ? '+' : ''}{(evaluation / 100).toFixed(2)}
            </span>
          </div>
          
          <div className="h-2 bg-terminal-bg border border-terminal-border overflow-hidden">
            <div 
              className="h-full bg-terminal-neon transition-all duration-500"
              style={{ width: evalBarWidth + '%' }}
            />
          </div>
          
          <div className="flex justify-between text-xs">
            <span className="text-terminal-muted">Black</span>
            <span className="text-terminal-muted">White</span>
          </div>
        </div>
      </div>
      
      {activeGame && (
        <div className="border border-terminal-border bg-terminal-surface p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-bold">Game Info</h3>
            <span className={
              activeGame.status === 'Active' 
                ? 'text-terminal-neon' 
                : 'text-terminal-muted'
            }>
              {activeGame.status}
            </span>
          </div>
          
          <div className="space-y-2 text-sm font-mono">
            <div className="flex justify-between">
              <span className="text-terminal-muted">White</span>
              <span className="text-xs">{activeGame.whitePlayer.slice(0, 12)}...</span>
            </div>
            <div className="flex justify-between">
              <span className="text-terminal-muted">Black</span>
              <span className="text-xs">{activeGame.blackPlayer.slice(0, 12)}...</span>
            </div>
            <div className="flex justify-between">
              <span className="text-terminal-muted">Moves</span>
              <span>{activeGame.moveCount}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
