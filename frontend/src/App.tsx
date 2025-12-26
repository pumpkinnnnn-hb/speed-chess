import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Header } from './components/Header';
import { ChessBoard } from './components/ChessBoard';
import { BettingPanel } from './components/BettingPanel';
import { GamesList } from './components/GamesList';
import { useEffect } from 'react';
import { useGameStore } from './store/gameStore';
import { useActiveGames } from './hooks/useGames';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

function AppContent() {
  const { data: games } = useActiveGames();
  const { setGames, setActiveGame, activeGameId } = useGameStore();

  useEffect(() => {
    if (games) {
      setGames(games);
      if (!activeGameId && games.length > 0) {
        setActiveGame(games[0].id);
      }
    }
  }, [games, setGames, setActiveGame, activeGameId]);

  return (
    <div className="min-h-screen bg-terminal-bg">
      <Header />

      <main className="container mx-auto px-6 py-8">
        <div className="grid grid-cols-12 gap-6">
          <div className="col-span-3">
            <GamesList />
          </div>

          <div className="col-span-6">
            <ChessBoard />
          </div>

          <div className="col-span-3">
            <BettingPanel />
          </div>
        </div>
      </main>

      <footer className="border-t border-terminal-border bg-terminal-surface/30 mt-12">
        <div className="container mx-auto px-6 py-4">
          <div className="flex items-center justify-between text-xs text-terminal-muted">
            <div>
              Built on <span className="neon-text">Linera</span> Blockchain
            </div>
            <div className="flex items-center gap-4">
              <span>Powered by Stockfish Oracle</span>
              <span>•</span>
              <span>Linera Buildathon Wave 5</span>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
    </QueryClientProvider>
  );
}
