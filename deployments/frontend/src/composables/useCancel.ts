import { ref } from "vue";
import {
  sendTransaction,
  getAccount,
  waitForTransactionReceipt,
  switchChain,
  getChainId,
} from "@wagmi/core";
import { Connection, Transaction, VersionedTransaction } from "@solana/web3.js";
import { wagmiConfig } from "../wallets";
import type { Wallet } from "ethers";
import type { Keypair } from "@solana/web3.js";
import type Solflare from "@solflare-wallet/sdk";
import type { TrackedOrder } from "./useOrders";
import { getQuoterUrl, type NetworkType } from "../config/network";

// Supported chain IDs matching wagmi config
type SupportedChainId = 1 | 8453 | 11155111 | 84532 | 421614;

// Solana chain IDs
const SOLANA_CHAIN_IDS = [1399811149, 1399811150];

export interface CancelResult {
  orderId: string;
  txHash: string;
}

export interface EvmTransaction {
  to: string;
  data: string;
  value: string;
}

export interface CancelResponse {
  order_id: string;
  evm_transaction?: EvmTransaction;
  svm_transaction?: string;
  orderbook_address: string;
  chain_id: number;
}

export type ChainType = "evm" | "svm";

export function useCancel() {
  const loading = ref(false);
  const error = ref<string | null>(null);

  /**
   * Determine chain type from chain ID
   */
  function getChainType(chainId: number): ChainType {
    return SOLANA_CHAIN_IDS.includes(chainId) ? "svm" : "evm";
  }

  /**
   * Ensure the wallet is connected to the correct chain, switching if necessary
   */
  async function ensureCorrectChain(targetChainId: number): Promise<void> {
    const currentChainId = getChainId(wagmiConfig);
    if (currentChainId !== targetChainId) {
      await switchChain(wagmiConfig, {
        chainId: targetChainId as SupportedChainId,
      });
    }
  }

  /**
   * Send a single EVM transaction
   */
  async function sendEvmTransaction(
    tx: EvmTransaction,
    localSigner?: Wallet | null,
    chainId?: number,
  ): Promise<string> {
    if (localSigner) {
      // Local mode - sign and send directly with ethers
      const result = await localSigner.sendTransaction({
        to: tx.to,
        data: tx.data,
        value: tx.value,
      });
      await result.wait();
      return result.hash;
    } else {
      // External wallet mode - use wagmi
      const account = getAccount(wagmiConfig);
      if (!account.address) {
        throw new Error("EVM wallet not connected");
      }

      // Switch to the correct chain if specified
      if (chainId) {
        await ensureCorrectChain(chainId);
      }

      const txHash = await sendTransaction(wagmiConfig, {
        to: tx.to as `0x${string}`,
        data: tx.data as `0x${string}`,
        value: BigInt(tx.value),
        chainId: chainId as SupportedChainId | undefined,
      });

      await waitForTransactionReceipt(wagmiConfig, { hash: txHash });
      return txHash;
    }
  }

  /**
   * Send an SVM transaction
   */
  async function sendSvmTransaction(
    transactionBase64: string,
    rpcUrl: string,
    localKeypair?: Keypair | null,
    solflareWallet?: Solflare | null,
  ): Promise<string> {
    const connection = new Connection(rpcUrl);
    const transactionBuffer = Buffer.from(transactionBase64, "base64");

    let txHash: string;

    if (localKeypair) {
      // Local mode - sign with keypair
      let transaction: Transaction | VersionedTransaction;
      try {
        transaction = Transaction.from(transactionBuffer);
        transaction.sign(localKeypair);
      } catch {
        transaction = VersionedTransaction.deserialize(transactionBuffer);
        transaction.sign([localKeypair]);
      }

      const signature = await connection.sendRawTransaction(
        transaction.serialize(),
      );
      txHash = signature;
    } else {
      // External wallet mode - use Solflare
      if (!solflareWallet || !solflareWallet.isConnected) {
        throw new Error("Solflare wallet not connected");
      }

      let transaction: Transaction | VersionedTransaction;
      try {
        transaction = Transaction.from(transactionBuffer);
      } catch {
        transaction = VersionedTransaction.deserialize(transactionBuffer);
      }

      const signedTransaction =
        await solflareWallet.signTransaction(transaction);
      const signature = await connection.sendRawTransaction(
        signedTransaction.serialize(),
      );
      txHash = signature;
    }

    // Confirm the transaction
    await connection.confirmTransaction(txHash, "confirmed");
    return txHash;
  }

  /**
   * Get cancel transaction from quoter API
   */
  async function getCancelTransaction(
    order: TrackedOrder,
    callerAddress: string,
    network: NetworkType,
  ): Promise<CancelResponse> {
    const quoterUrl = getQuoterUrl(network);

    const request = {
      order_id: order.order_id.startsWith("0x")
        ? order.order_id
        : `0x${order.order_id}`,
      origin_chain_id: order.origin_chain_id,
      dest_chain_id: order.dest_chain_id,
      version: order.version,
      nonce: order.nonce,
      created_at: order.created_at,
      fill_deadline: order.fill_deadline,
      sender: order.sender,
      recipient: order.recipient,
      token_in: order.token_in,
      token_out: order.token_out,
      amount_in: parseInt(order.amount_in),
      amount_out: parseInt(order.amount_out),
      solver: order.solver,
      caller_address: callerAddress,
    };

    const response = await fetch(`${quoterUrl}/cancel`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        errorData.error || `Failed to get cancel transaction: ${response.status}`,
      );
    }

    return response.json();
  }

  /**
   * Cancel an order
   */
  async function cancelOrder(
    order: TrackedOrder,
    network: NetworkType,
    callerAddress: string,
    options: {
      localEvmSigner?: Wallet | null;
      localSvmKeypair?: Keypair | null;
      solflareWallet?: Solflare | null;
      svmRpcUrl?: string;
    },
  ): Promise<CancelResult> {
    loading.value = true;
    error.value = null;

    try {
      // Get cancel transaction from quoter
      const cancelResponse = await getCancelTransaction(
        order,
        callerAddress,
        network,
      );

      const chainType = getChainType(order.dest_chain_id);
      let txHash: string;

      if (chainType === "evm") {
        if (!cancelResponse.evm_transaction) {
          throw new Error("No EVM transaction data returned");
        }
        txHash = await sendEvmTransaction(
          cancelResponse.evm_transaction,
          options.localEvmSigner,
          cancelResponse.chain_id,
        );
      } else {
        if (!cancelResponse.svm_transaction) {
          throw new Error("No SVM transaction data returned");
        }
        if (!options.svmRpcUrl) {
          throw new Error("SVM RPC URL required for Solana transactions");
        }
        txHash = await sendSvmTransaction(
          cancelResponse.svm_transaction,
          options.svmRpcUrl,
          options.localSvmKeypair,
          options.solflareWallet,
        );
      }

      return {
        orderId: cancelResponse.order_id,
        txHash,
      };
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : "Failed to cancel order";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    error,
    cancelOrder,
    getChainType,
  };
}
