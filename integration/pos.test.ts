import { getTestSetup } from './init';
import { Interface } from 'ethers';
import {
  BuildTransactionRequest,
  BuildTransactionResponse,
  BuildTransactionErrorResponse,
  EvmTransactionParams,
  CheckTransactionRequest,
  CheckTransactionResponse,
  SolanaTransactionParams
} from './types/pos';


describe('POS', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const fromAddress = '0x2aae531a81461f029cd55cb46703211c9227ba05';
  const baseFromAddress = `eip155:8453:${fromAddress}`;

  const toAddress = '0x2aae531a81461f029cd55cb46703211c9227ba06';
  const baseToAddress = `eip155:8453:${toAddress}`;

  const baseUSDCContractAddress = '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913';
  const baseUSDC = `eip155:8453/erc20:${baseUSDCContractAddress}`;

  const baseNative = 'eip155:8453/slip44:60';

  const usdcAmount = '0.001';
  const usdcAmountBigInt = BigInt(1000);
  const nativeAmount = '0.000001';
  const nativeAmountHex = '0xe8d4a51000'

  const unsupportedAsset = 'eip155:999/erc20:0x1234567890123456789012345678901234567890';

  const unsupportedNamespace = 'someNamespace:1';
  const unsupportedSender = `${unsupportedNamespace}:0x1234567890123456789012345678901234567890`;
  const unsupportedRecipient = `${unsupportedNamespace}:0x1234567890123456789012345678901234567891`;
  const unsupportedNamespaceAsset = `${unsupportedNamespace}:0x1234567890123456789012345678901234567892`;


  const txIdBaseSepolia = 'djF8ZWlwMTU1Ojg0NTMyfGZlZjMzYjE0LTlhMmQtNDZhMC1hYTk5LThmOTY1OGQwNzc2Nw'
  const confirmedTxId = '0x5005606b67977f2641b29f4c03b05473a11b2f7f5709e4b6a38d442bc356a5e9' 
  const erc20Interface = new Interface([
    'function transfer(address to, uint256 amount)',
  ]);


  // SOLANA
  const solanaMainnetChainId = 'solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp';
  const solanaUsdcAsset = `${solanaMainnetChainId}/token:EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
  const solanaMainnetSender = `7VHUFJHWu2CuExkJcJrzhQPJ2oygupTWkL2A2For4BmE`
  const solanaMainnetRecipient = `5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9`
  const solanaMainnetSenderCaip10 = `${solanaMainnetChainId}:${solanaMainnetSender}`
  const solanaMainnetRecipientCaip10 = `${solanaMainnetChainId}:${solanaMainnetRecipient}`
  const solanaMainnetAmount = '0.001'

  const solanaDevnetTransactionId = 'djF8c29sYW5hOkV0V1RSQUJaYVlxNmlNZmVZS291UnUxNjZWVTJ4cWExfDE0MjkxMTM0LTEzMDUtNDZlOS04NDMyLTZhZjI4ZjYwODQyYQ'
  const solanaDevnetSignature = '2SCP4z9Bs2WEcBZZzwH812HNoBJKGcGz2d41UmSvp2QBaQ9BeqPqybgsiTn9LVtYnKNqJTFctWQbrGvgW7J7WxHV'

  describe('EVM', () => {
    it('should build an ERC20 transfer transaction', async () => {
      const payload: BuildTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_pos_buildTransaction',
        params: {
          asset: baseUSDC,
          amount: usdcAmount,
          recipient: baseToAddress,
          sender: baseFromAddress,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      const result = responseData.result;
      expect(result).toBeDefined();
      expect(result.id).toBeDefined();
      expect(result.id.length).toBeGreaterThan(10);
      expect(result.transactionRpc).toBeDefined();
      expect(result.transactionRpc.method).toBe('eth_sendTransaction')
      const params: EvmTransactionParams = result.transactionRpc.params[0];
      expect(params).toBeDefined();
      expect(params.to).toBe(baseUSDCContractAddress.toLowerCase());
      expect(params.from).toBe(fromAddress.toLowerCase());
      expect(params.value).toBe('0x0');
      expect(params.input).toBeDefined();
      expect(params.data).toBeDefined();
      expect(params.data).toBe(params.input);
      expect(params.input?.length).toBeGreaterThan(0);
      const decodedData = erc20Interface.decodeFunctionData('transfer', params.input || '');
      expect(decodedData[0].toLowerCase()).toBe(toAddress.toLowerCase());
      expect(decodedData[1]).toBe(usdcAmountBigInt);
    });

    it('should build a native transaction', async () => {
      const payload: BuildTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_pos_buildTransaction',
        params: {
          asset: baseNative,
          amount: nativeAmount,
          recipient: baseToAddress,
          sender: baseFromAddress,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      const result = responseData.result;
      expect(result).toBeDefined();
      expect(result.id).toBeDefined();
      expect(result.id.length).toBeGreaterThan(10);
      expect(result.transactionRpc).toBeDefined();
      expect(result.transactionRpc.method).toBe('eth_sendTransaction')
      const params: EvmTransactionParams = result.transactionRpc.params[0];
      expect(params).toBeDefined();
      expect(params.to).toBe(toAddress.toLowerCase());
      expect(params.from).toBe(fromAddress.toLowerCase());
      expect(params.value).toBe(nativeAmountHex);
      expect(params.input).toBeUndefined();
    });


    it('should not build a transaction with a recipient that is not a valid address', async () => {
      const payload = {
        jsonrpc: '2.0' as const,
        id: 1,
        method: 'reown_pos_buildTransaction' as const,
        params: {
          asset: baseUSDC,
          amount: usdcAmount,
          recipient: '0x1234567890123456789012345678901234567890',
          sender: baseFromAddress,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(400);
      const errorResponse = response.data as BuildTransactionErrorResponse;
      expect(errorResponse.error.message.includes('Invalid Recipient')).toBe(true);
    });

    it('should not build a transaction with an invalid asset', async () => {
      const payload = {
        jsonrpc: '2.0' as const,
        id: 1,
        method: 'reown_pos_buildTransaction' as const,
        params: {
          asset: unsupportedAsset,
          amount: usdcAmount,
          recipient: baseToAddress,
          sender: baseFromAddress,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(400);
      const errorResponse = response.data as BuildTransactionErrorResponse;
      expect(errorResponse.error.message.includes('Validation error')).toBe(true);
    });

    it('should not build a transaction with an invalid namespace', async () => {
      const payload = {
        jsonrpc: '2.0' as const,
        id: 1,
        method: 'reown_pos_buildTransaction' as const,
        params: {
          asset: unsupportedNamespaceAsset,
          amount: usdcAmount,
          recipient: unsupportedRecipient,
          sender: unsupportedSender,
        }
      };
      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(400);
      const errorResponse = response.data as BuildTransactionErrorResponse;
      expect(errorResponse.error.message.includes('Validation error')).toBe(true);
    });


    it('should check the transaction status', async () => {
      const payload: CheckTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_pos_checkTransaction',
        params: {
          id: txIdBaseSepolia,
          sendResult: confirmedTxId,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as CheckTransactionResponse;
      expect(responseData.result).toBeDefined();
      expect(responseData.result.status).toBe('CONFIRMED');
    })
  });


  describe('Solana', () => {
    it('should build a Solana transfer transaction', async () => {
      const payload: BuildTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_pos_buildTransaction',
        params: {
          asset: solanaUsdcAsset,
          amount: solanaMainnetAmount,
          recipient: solanaMainnetRecipientCaip10,
          sender: solanaMainnetSenderCaip10,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      const result = responseData.result;
      expect(result).toBeDefined();
      expect(result.id).toBeDefined();
      expect(result.id.length).toBeGreaterThan(10);
      expect(result.transactionRpc).toBeDefined();
      expect(result.transactionRpc.method).toBe('solana_signAndSendTransaction')
      const params = result.transactionRpc.params as SolanaTransactionParams; 
      expect(params).toBeDefined();
      expect(params.transaction).toBeDefined();
      expect(params.transaction.length).toBeGreaterThan(0);
      expect(params.pubkey).toBe(solanaMainnetSender);
    });


    it('should check the transaction status', async () => {
      const payload: CheckTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_pos_checkTransaction',
        params: {
          id: solanaDevnetTransactionId,
          sendResult: solanaDevnetSignature,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as CheckTransactionResponse;
      expect(responseData.result).toBeDefined();
      expect(responseData.result.status).toBe('CONFIRMED');
    });
    
  });
});