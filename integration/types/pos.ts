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

export interface BaseBuildTransactionParams {
  asset: string;
  amount: string;
  recipient: string;
  sender: string;
}

export interface EvmBuildTransactionParams extends BaseBuildTransactionParams {
  asset: string;
  recipient: string;
  sender: string;
}

export type BuildTransactionParams = EvmBuildTransactionParams;

export type CheckTransactionParams = {
  id: string;
  txid: string;
};

export interface BuildTransactionRequest extends BaseJsonRpcRequest {
  method: 'reown_pos_buildTransaction';
  params: BuildTransactionParams;
}
export interface CheckTransactionRequest extends BaseJsonRpcRequest {
  method: 'reown_pos_checkTransaction';
  params: CheckTransactionParams;
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

export interface BaseTransactionRpc {
  method: string;
  params: unknown[];
}

export interface EvmTransactionRpc extends BaseTransactionRpc {
  method: 'eth_sendTransaction';
  params: [EvmTransactionParams];
}

export type TransactionRpc = EvmTransactionRpc;

export interface BuildTransactionResult {
  id: string;
  transactionRpc: TransactionRpc;
}

export interface CheckTransactionResult {
  status: string;
  checkIn: number;
}

export type BuildTransactionResponse = BaseJsonRpcResponse<BuildTransactionResult>;

export type CheckTransactionResponse = BaseJsonRpcResponse<CheckTransactionResult>;

export type BuildTransactionErrorResponse = BaseJsonRpcResponse<never> & {
  result?: never;
  error: JsonRpcError;
};


export type PosResponse = BuildTransactionResponse | BuildTransactionErrorResponse;
