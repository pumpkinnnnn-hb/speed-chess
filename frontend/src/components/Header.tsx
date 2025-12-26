import { useGameStore } from '@/store/gameStore';

export function Header() {
  const { wallet, setWallet } = useGameStore();

  const handleConnectWallet = async () => {
    try {
      // For Linera, we use the chain ID from env
      const chainId = import.meta.env.VITE_CHAIN_ID;

      // Mock wallet connection - in real app would query actual balance
      setWallet({
        chainId: chainId,
        balance: 1000000000, // 1000 tokens (6 decimals)
        isConnected: true,
      });
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    }
  };

  const formatBalance = (balance: number) => {
    return (balance / 1_000_000).toFixed(2);
  };

  const shortenAddress = (address: string) => {
    if (!address) return '';
    const start = address.slice(0, 8);
    const end = address.slice(-6);
    return start + '...' + end;
  };
  
  return (
    <header className="border-b border-terminal-border bg-terminal-surface/50 backdrop-blur-sm">
      <div className="container mx-auto px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="text-2xl">♔</div>
            <div>
              <h1 className="text-xl font-bold tracking-tight neon-text">
                SPEED.CHESS
              </h1>
              <p className="text-xs text-terminal-muted">
                Live Betting Protocol
              </p>
            </div>
          </div>
          
          <div className="flex items-center gap-4">
            {wallet.isConnected ? (
              <>
                <div className="text-right">
                  <div className="text-xs text-terminal-muted">Balance</div>
                  <div className="text-lg font-bold neon-text">
                    {formatBalance(wallet.balance)} <span className="text-sm">TOKENS</span>
                  </div>
                </div>
                
                <div className="h-10 w-px bg-terminal-border" />
                
                <div className="terminal-button px-4 py-2 cursor-default">
                  <div className="text-xs text-terminal-muted mb-1">Connected</div>
                  <div className="font-mono text-xs">
                    {shortenAddress(wallet.chainId)}
                  </div>
                </div>
              </>
            ) : (
              <button
                className="terminal-button-primary"
                onClick={handleConnectWallet}
              >
                Connect Wallet
              </button>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
