import { useState, useEffect } from 'react';

// Chain IDs from the fresh network wallet
const DEFAULT_CHAIN_ID = import.meta.env.VITE_CHAIN_ID || '8974e56566be0e114121934122b0d867123b1d366a815f5c6104e37c9ae735f8';
const PLAYER2_CHAIN_ID = import.meta.env.VITE_PLAYER2_CHAIN || '93b79b297357c49594fd2f0d3f8672d179b0c19411711bfdc2f09d5c53c39932';

// Real owner public keys from the wallet (these have private keys in the keystore)
const CHAIN_OWNERS: Record<string, string> = {
  '8974e56566be0e114121934122b0d867123b1d366a815f5c6104e37c9ae735f8': '0x13a57a27bce4761beeebd6b6273f71dd1bdcdca977528f5e84742bdf3b41ba70',
  '93b79b297357c49594fd2f0d3f8672d179b0c19411711bfdc2f09d5c53c39932': '0xe92fb3431b802f3bcf315a81d8e3a5101971ec635ffbf106fd60ef524cb28c1a',
  '129612bfda6537ea6978dbc3163dbe8df63f1b2137fe946b68cba4fcdebb6aae': '0xc63f11e5fc14d91b2842f5139dfc28ff976c42b72b1845d54c96e16c6e230c33',
};

interface WalletState {
  chainId: string | null;
  publicKey: string | null;
  isLoading: boolean;
  error: string | null;
}

export function useWallet() {
  const [state, setState] = useState<WalletState>({
    chainId: null,
    publicKey: null,
    isLoading: true,
    error: null,
  });

  useEffect(() => {
    const initWallet = async () => {
      try {
        console.log('[WALLET] Initializing with server-side signing...');

        // Check if we already have a chain in localStorage
        let savedChainId = localStorage.getItem('linera_chain_id');

        // Validate that saved chain is one of our known chains
        if (savedChainId && !CHAIN_OWNERS[savedChainId]) {
          console.log('[WALLET] Clearing stale chain data from old network');
          localStorage.removeItem('linera_chain_id');
          localStorage.removeItem('linera_public_key');
          savedChainId = null;
        }

        if (savedChainId) {
          const publicKey = CHAIN_OWNERS[savedChainId];
          console.log('[WALLET] Using saved chain:', savedChainId.slice(0, 16) + '...');
          setState({
            chainId: savedChainId,
            publicKey,
            isLoading: false,
            error: null,
          });
          return;
        }

        // Use default chain for this user session
        // In a real multiplayer setup, different browser tabs/users would get different chains
        const chainId = DEFAULT_CHAIN_ID;
        const publicKey = CHAIN_OWNERS[chainId];

        console.log('[WALLET] Using default chain:', chainId.slice(0, 16) + '...');
        console.log('[WALLET] Owner public key:', publicKey.slice(0, 20) + '...');

        // Save to localStorage
        localStorage.setItem('linera_chain_id', chainId);
        localStorage.setItem('linera_public_key', publicKey);

        setState({
          chainId,
          publicKey,
          isLoading: false,
          error: null,
        });

        console.log('[WALLET] Initialized successfully with server-side signing');
      } catch (error) {
        console.error('[WALLET] Failed to initialize:', error);
        setState({
          chainId: null,
          publicKey: null,
          isLoading: false,
          error: error instanceof Error ? error.message : 'Failed to initialize wallet',
        });
      }
    };

    initWallet();
  }, []);

  const resetWallet = () => {
    localStorage.removeItem('linera_chain_id');
    localStorage.removeItem('linera_public_key');
    setState({
      chainId: null,
      publicKey: null,
      isLoading: true,
      error: null,
    });
    // Re-initialize
    window.location.reload();
  };

  // Switch to a different chain (for multiplayer testing)
  const switchToChain = (chainId: string) => {
    if (!CHAIN_OWNERS[chainId]) {
      console.error('[WALLET] Unknown chain:', chainId);
      return;
    }
    const publicKey = CHAIN_OWNERS[chainId];
    localStorage.setItem('linera_chain_id', chainId);
    localStorage.setItem('linera_public_key', publicKey);
    setState({
      chainId,
      publicKey,
      isLoading: false,
      error: null,
    });
    console.log('[WALLET] Switched to chain:', chainId.slice(0, 16) + '...');
  };

  // Get the opponent chain for multiplayer
  const getOpponentChain = (): string => {
    if (state.chainId === DEFAULT_CHAIN_ID) {
      return PLAYER2_CHAIN_ID;
    }
    return DEFAULT_CHAIN_ID;
  };

  return {
    ...state,
    resetWallet,
    switchToChain,
    getOpponentChain,
    availableChains: Object.keys(CHAIN_OWNERS),
  };
}
