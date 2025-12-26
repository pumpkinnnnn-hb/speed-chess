/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_LINERA_GRAPHQL_URL: string;
  readonly VITE_CHAIN_ID: string;
  readonly VITE_GAME_APP_ID: string;
  readonly VITE_BETTING_APP_ID: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
