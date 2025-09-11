export interface BaseJsonRpcRequest {
  jsonrpc: '2.0';
  id: number | string;
}

export interface BaseJsonRpcResponse<T = unknown> {
  id: number | string;
  jsonrpc: '2.0';
  result?: T;
  error?: JsonRpcError;
}

export interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

export interface PaymentIntent {
  asset: string;
  amount: string;
  recipient: string;
  sender: string;
}

export interface BuildTransactionParams {
  paymentIntents: PaymentIntent[];
  capabilities?: unknown;
}

export type CheckTransactionParams = {
  id: string;
  sendResult: string;
};

export interface BuildTransactionRequest extends BaseJsonRpcRequest {
  method: 'wc_pos_buildTransactions';
  params: BuildTransactionParams;
}
export interface CheckTransactionRequest extends BaseJsonRpcRequest {
  method: 'wc_pos_checkTransaction';
  params: CheckTransactionParams;
}

export interface SolanaTransactionParams {
  transaction: string;
  pubkey: string;
}
export interface EvmTransactionParams {
  from: string;
  to: string;
  value: string;
  gas?: string;
  gasPrice?: string;
  maxFeePerGas?: string;
  maxPriorityFeePerGas?: string;
  input?: string;
  data?: string;
}



export interface TronTransactionObject {
  raw_data: unknown;
  raw_data_hex: string;
  signature: string[] | null;
  txID: string;
  visible: boolean | undefined;
}

export interface TronTransaction {
  result: {
    result: boolean;
  };
  transaction: TronTransactionObject;
}

export interface TronTransactionParams {
  transaction: TronTransaction;
  address: string;
}

export interface BaseTransactionRpc {
  id: string;
  chainId: string;
  method: string;
  params: unknown;
}

export interface EvmTransactionRpc extends BaseTransactionRpc {
  method: 'eth_sendTransaction';
  params: [EvmTransactionParams];
}

export interface SolanaTransactionRpc extends BaseTransactionRpc {
  method: 'solana_signAndSendTransaction';
  params: SolanaTransactionParams;
}

export interface TronTransactionRpc extends BaseTransactionRpc {
  method: 'tron_signTransaction';
  params: TronTransactionParams;
}

export type TransactionRpc = EvmTransactionRpc | SolanaTransactionRpc | TronTransactionRpc;

export interface BuildTransactionResult {
  transactions: TransactionRpc[];
}

export interface CheckTransactionResult {
  status: string;
  checkIn?: number;
}

export type BuildTransactionResponse = BaseJsonRpcResponse<BuildTransactionResult>;

export type CheckTransactionResponse = BaseJsonRpcResponse<CheckTransactionResult>;

export type BuildTransactionErrorResponse = BaseJsonRpcResponse<never> & {
  result?: never;
  error: JsonRpcError;
};


export type PosResponse = BuildTransactionResponse | BuildTransactionErrorResponse;

// Supported Networks
export interface SupportedNetworksRequest extends BaseJsonRpcRequest {
  method: 'wc_pos_supportedNetworks';
  params: Record<string, never>;
}

export interface SupportedNamespaceInfo {
  name: string;
  methods: string[];
  events: string[];
  capabilities: unknown | null;
  assetNamespaces: string[];
}

export interface SupportedNetworksResult {
  namespaces: SupportedNamespaceInfo[];
}

export type SupportedNetworksResponse = BaseJsonRpcResponse<SupportedNetworksResult>;
