import { ref, onMounted } from 'vue'

export interface Asset {
  ticker: string
  symbol: string
  name: string
  icon: string
  address: string
  decimals: number
  chainId: number
  chain: string
  runtime: 'evm' | 'svm'
}

export function useAssets() {
  const assets = ref<Asset[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const quoterUrl = import.meta.env.VITE_QUOTER_URL || 'http://localhost:3000'

  async function fetchAssets(): Promise<Asset[]> {
    loading.value = true
    error.value = null

    try {
      const response = await fetch(`${quoterUrl}/assets`)

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }

      const data = await response.json()
      // Flatten assets: API returns chain_ids array, we need one entry per chainId
      const flattenedAssets: Asset[] = []
      for (const item of data) {
        const chainIds = item.chain_ids as number[]
        for (const chainId of chainIds) {
          flattenedAssets.push({
            ticker: item.ticker as string,
            symbol: item.ticker as string,
            name: item.name as string,
            icon: item.icon as string,
            address: item.address as string,
            decimals: item.decimals as number,
            chainId,
            chain: getChainName(chainId),
            runtime: isSolanaChainId(chainId) ? 'svm' : 'evm',
          })
        }
      }
      assets.value = flattenedAssets
      return assets.value
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch assets'
      return []
    } finally {
      loading.value = false
    }
  }

  function isSolanaChainId(chainId: number): boolean {
    // Solana mainnet, devnet, and local chain IDs
    return chainId === 1399811149 || chainId === 1399811150
  }

  function getChainName(chainId: number): string {
    const chainNames: Record<number, string> = {
      1: 'ethereum',
      8453: 'base',
      42161: 'arbitrum',
      11155111: 'sepolia',
      84532: 'base-sepolia',
      1399811149: 'solana',
      1399811150: 'solana-devnet',
    }
    return chainNames[chainId] || 'unknown'
  }

  function getAssetsForChain(chainId: number): Asset[] {
    return assets.value.filter(asset => asset.chainId === chainId)
  }

  function getAssetForChain(ticker: string, chainId: number): Asset | undefined {
    return assets.value.find(asset => asset.ticker === ticker && asset.chainId === chainId)
  }

  function getUniqueTickers(): string[] {
    const tickers = new Set(assets.value.map(asset => asset.ticker))
    return Array.from(tickers)
  }

  onMounted(() => {
    fetchAssets()
  })

  return {
    assets,
    loading,
    error,
    fetchAssets,
    getAssetsForChain,
    getAssetForChain,
    getUniqueTickers,
  }
}
