import { createConfig, http } from "@wagmi/core"
import { mainnet, sepolia } from "@wagmi/core/chains"
import { injected } from "@wagmi/connectors"
import Solflare from "@solflare-wallet/sdk"

// Define local Anvil chain
const anvilLocal = {
  id: 31337,
  name: 'Anvil Local',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: {
    default: { http: ['http://localhost:8545'] },
  },
} as const

// Define Base chain
const base = {
  id: 8453,
  name: 'Base',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: {
    default: { http: ['https://mainnet.base.org'] },
  },
} as const

// Define local Base chain
const baseLocal = {
  id: 31338,
  name: 'Base Local',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: {
    default: { http: ['http://localhost:8546'] },
  },
} as const

// Wagmi config for EVM wallet connections
// Uses injected connector for browser extension wallets (Rabby, MetaMask, etc.)
export const wagmiConfig = createConfig({
  chains: [mainnet, sepolia, base, anvilLocal, baseLocal],
  connectors: [
    injected({
      shimDisconnect: true,
    }),
  ],
  transports: {
    [mainnet.id]: http(),
    [sepolia.id]: http(),
    [base.id]: http(),
    [anvilLocal.id]: http(import.meta.env.VITE_ANVIL_RPC || 'http://localhost:8545'),
    [baseLocal.id]: http(import.meta.env.VITE_BASE_LOCAL_RPC || 'http://localhost:8546'),
  },
})

// Solflare instance (singleton) for Solana wallet connections
export const solflare = new Solflare()
