import { getTestSetup } from './init';
import { Interface } from 'ethers';
import {
  BuildTransactionRequest,
  BuildTransactionResponse,
  BuildTransactionErrorResponse,
  EvmTransactionParams,
  EvmTransactionRpc,
  CheckTransactionRequest,
  CheckTransactionResponse,
  SolanaTransactionParams,
  TronTransactionParams,
  SolanaTransactionRpc
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

  // TRON
  const tronNileTestnetChainId = 'tron:0xcd8690dc'
  const tronUsdtAsset = `${tronNileTestnetChainId}/trc20:TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf`
  const tronNileTestnetSender = 'TDGSR64oU4QDpViKfdwawSiqwyqpUB6JUD'
  const tronNileTestnetRecipient = 'TKGRE6oiU3rEzasue4MsB6sCXXSTx9BAe3'
  const tronNileTestnetSenderCaip10 = `${tronNileTestnetChainId}:${tronNileTestnetSender}`
  const tronNileTestnetRecipientCaip10 = `${tronNileTestnetChainId}:${tronNileTestnetRecipient}`
  const tronNileTestnetAmount = '0.001'

  const tronNileTestnetTransactionId = 'djF8dHJvbjoweGNkODY5MGRjfDQwZDEzZjMyLTUxNmYtNDgyMC05N2QyLWZlMTljMjJiNzk3OQ'
  const tronNileTestnetSendResult= "{\"raw_data\":{\"contract\":[{\"parameter\":{\"type_url\":\"type.googleapis.com/protocol.TriggerSmartContract\",\"value\":{\"contract_address\":\"41eca9bc828a3005b9a3b909f2cc5c2a54794de05f\",\"data\":\"a9059cbb000000000000000000000000250bcf7ea20ca27ad3fa7577d63d1a7ccb44363700000000000000000000000000000000000000000000000000000000000f4240\",\"owner_address\":\"417c972ef270213301735b0039e8405dbabf91356c\"}},\"type\":\"TriggerSmartContract\"}],\"expiration\":1757072835000,\"fee_limit\":2637000,\"ref_block_bytes\":\"e2b0\",\"ref_block_hash\":\"6c5fe40810979f92\",\"timestamp\":1757072775461},\"raw_data_hex\":\"0a02e2b022086c5fe40810979f9240b883cfcd91335aae01081f12a9010a31747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e54726967676572536d617274436f6e747261637412740a15417c972ef270213301735b0039e8405dbabf91356c121541eca9bc828a3005b9a3b909f2cc5c2a54794de05f2244a9059cbb000000000000000000000000250bcf7ea20ca27ad3fa7577d63d1a7ccb44363700000000000000000000000000000000000000000000000000000000000f424070a5b2cbcd91339001c8f9a001\",\"signature\":[\"1ec03374b4a5ee0579aca0c029022433fb7558be5e2405d54c9cb29523fa980740b6f8f77398b6ddb7fcf86583f908fdf4b837d3497cd12f5d76cb2913e576a301\"],\"txID\":\"3673cd659cbb7ef8c14e3972eb03d194d43db9a53d7cb3df31c1cf9194864c76\",\"visible\":false}"

  describe('EVM', () => {
    it('should build an ERC20 transfer transaction', async () => {
      const payload: BuildTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'wc_pos_buildTransactions',
        params: {
          paymentIntents: [
            {
              asset: baseUSDC,
              amount: usdcAmount,
              recipient: baseToAddress,
              sender: baseFromAddress,
            }
          ]
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      const result = responseData.result;
      expect(result).toBeDefined();
      expect(result.transactions).toBeDefined();
      expect(result.transactions.length).toBe(1);
      const tx = result.transactions[0];
      expect(tx.id).toBeDefined();
      expect(tx.id.length).toBeGreaterThan(10);
      expect(tx.method).toBe('eth_sendTransaction')
      const params: EvmTransactionParams = (tx as EvmTransactionRpc).params[0];
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
        method: 'wc_pos_buildTransactions',
        params: {
          paymentIntents: [
            {
              asset: baseNative,
              amount: nativeAmount,
              recipient: baseToAddress,
              sender: baseFromAddress,
            }
          ]
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      const result = responseData.result;
      expect(result).toBeDefined();
      expect(result.transactions).toBeDefined();
      expect(result.transactions.length).toBe(1);
      const tx = result.transactions[0];
      expect(tx.id).toBeDefined();
      expect(tx.id.length).toBeGreaterThan(10);
      expect(tx.method).toBe('eth_sendTransaction')
      const params: EvmTransactionParams = (tx as EvmTransactionRpc).params[0];
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
        method: 'wc_pos_buildTransactions' as const,
        params: {
          paymentIntents: [
            {
              asset: baseUSDC,
              amount: usdcAmount,
              recipient: '0x1234567890123456789012345678901234567890',
              sender: baseFromAddress,
            }
          ]
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
        method: 'wc_pos_buildTransactions' as const,
        params: {
          paymentIntents: [
            {
              asset: unsupportedAsset,
              amount: usdcAmount,
              recipient: baseToAddress,
              sender: baseFromAddress,
            }
          ]
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
        method: 'wc_pos_buildTransactions' as const,
        params: {
          paymentIntents: [
            {
              asset: unsupportedNamespaceAsset,
              amount: usdcAmount,
              recipient: unsupportedRecipient,
              sender: unsupportedSender,
            }
          ]
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
        method: 'wc_pos_checkTransaction',
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
        method: 'wc_pos_buildTransactions',
        params: {
          paymentIntents: [
            {
              asset: solanaUsdcAsset,
              amount: solanaMainnetAmount,
              recipient: solanaMainnetRecipientCaip10,
              sender: solanaMainnetSenderCaip10,
            }
          ]
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      const result = responseData.result;
      expect(result).toBeDefined();
      expect(result.transactions).toBeDefined();
      expect(result.transactions.length).toBe(1);
      const tx = result.transactions[0];
      expect(tx.id).toBeDefined();
      expect(tx.id.length).toBeGreaterThan(10);
      expect(tx.method).toBe('solana_signAndSendTransaction')
      const params = tx.params as SolanaTransactionParams; 
      expect(params).toBeDefined();
      expect(params.transaction).toBeDefined();
      expect(params.transaction.length).toBeGreaterThan(0);
      expect(params.pubkey).toBe(solanaMainnetSender);
    });


    it('should check the transaction status', async () => {
      const payload: CheckTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'wc_pos_checkTransaction',
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

  describe('Tron', () => {
    it('should build a Tron transfer transaction', async () => {
      const payload: BuildTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'wc_pos_buildTransactions',
        params: {
          paymentIntents: [
            {
              asset: tronUsdtAsset,
              amount: tronNileTestnetAmount,
              recipient: tronNileTestnetRecipientCaip10,
              sender: tronNileTestnetSenderCaip10,
            }
          ]
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);

      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      const result = responseData.result;
      expect(result).toBeDefined();
      expect(result.transactions).toBeDefined();
      expect(result.transactions.length).toBe(1);
      const tx = result.transactions[0];
      expect(tx.id).toBeDefined();
      expect(tx.id.length).toBeGreaterThan(10);
      expect(tx.method).toBe('tron_signTransaction')
      const params = tx.params as TronTransactionParams; 
      expect(params).toBeDefined();
      expect(params.address ).toBeDefined();
      expect(params.transaction).toBeDefined();
      expect(params.transaction.transaction).toBeDefined();
      expect(params.transaction.transaction.raw_data_hex).toBeDefined();
      expect(params.transaction.transaction.raw_data_hex.length).toBeGreaterThan(0);
      expect(params.transaction.transaction.signature).toBeNull();
      expect(params.transaction.transaction.txID).toBeDefined();
      expect(params.transaction.transaction.txID.length).toBeGreaterThan(0);
      expect(params.transaction.transaction.visible).toBeDefined();
      expect(params.transaction.transaction.visible).toBe(false);
    });
  });

  describe('Tron', () => {
    it('should check the transaction status', async () => {
      const payload: CheckTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'wc_pos_checkTransaction',
        params: {
  
          id: tronNileTestnetTransactionId,
          sendResult: tronNileTestnetSendResult,
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);
      console.log(response.data);
      expect(response.status).toBe(200);
      const responseData = response.data as CheckTransactionResponse;
      expect(responseData.result).toBeDefined();
      expect(responseData.result.status).toBe('CONFIRMED');
    });
  });

  describe('Multiple transactions', () => {
    it('should build multiple evm transactions', async () => {
      const payload: BuildTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'wc_pos_buildTransactions',
        params: {
          paymentIntents: [
            {
              asset: baseUSDC,
              amount: usdcAmount,
              recipient: baseToAddress,
              sender: baseFromAddress,
            },
            {
              asset: baseUSDC,
              amount: usdcAmount,
              recipient: baseToAddress,
              sender: baseFromAddress,
            }
          ]
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);
      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      expect(responseData.result).toBeDefined();
      expect(responseData.result.transactions).toBeDefined();
      expect(responseData.result.transactions.length).toBe(2);
      const tx = responseData.result.transactions[0];
      expect(tx.id).toBeDefined();
      expect(tx.id.length).toBeGreaterThan(10);
      expect(tx.method).toBe('eth_sendTransaction')
      const params: EvmTransactionParams = (tx as EvmTransactionRpc).params[0];
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
      const tx2 = responseData.result.transactions[1];
      expect(tx2.id).toBeDefined();
      expect(tx2.id.length).toBeGreaterThan(10);
      expect(tx2.method).toBe('eth_sendTransaction')
      const params2: EvmTransactionParams = (tx2 as EvmTransactionRpc).params[0];
      expect(params2).toBeDefined();
      expect(params2.to).toBe(baseUSDCContractAddress.toLowerCase());
      expect(params2.from).toBe(fromAddress.toLowerCase());
      expect(params2.value).toBe('0x0');
      expect(params2.input).toBeDefined();
      expect(params2.data).toBeDefined();
      expect(params2.data).toBe(params2.input);
      expect(params2.input?.length).toBeGreaterThan(0);
      const decodedData2 = erc20Interface.decodeFunctionData('transfer', params2.input || '');
      expect(decodedData2[0].toLowerCase()).toBe(toAddress.toLowerCase());
      expect(decodedData2[1]).toBe(usdcAmountBigInt);
    });

    it('should build multiple transactions on different chains', async () => {
      const payload: BuildTransactionRequest = {
        jsonrpc: '2.0',
        id: 1,
        method: 'wc_pos_buildTransactions',
        params: {
          paymentIntents: [
            {
              asset: baseUSDC,
              amount: usdcAmount,
              recipient: baseToAddress,
              sender: baseFromAddress,
            },
            {
              asset: solanaUsdcAsset,
              amount: solanaMainnetAmount,
              recipient: solanaMainnetRecipientCaip10,
              sender: solanaMainnetSenderCaip10,
            }
          ]
        }
      };

      const response = await httpClient.post(`${baseUrl}/v1/json-rpc?projectId=${projectId}`, payload);
      expect(response.status).toBe(200);
      const responseData = response.data as BuildTransactionResponse;
      expect(responseData.result).toBeDefined();
      expect(responseData.result.transactions).toBeDefined();
      expect(responseData.result.transactions.length).toBe(2);
      const tx = responseData.result.transactions[0];
      expect(tx.id).toBeDefined();
      expect(tx.id.length).toBeGreaterThan(10);
      expect(tx.method).toBe('eth_sendTransaction')
      const params: EvmTransactionParams = (tx as EvmTransactionRpc).params[0];
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
      const tx2 = responseData.result.transactions[1];
      expect(tx2.id).toBeDefined();
      expect(tx2.id.length).toBeGreaterThan(10);
      expect(tx2.method).toBe('solana_signAndSendTransaction')
      const params2: SolanaTransactionParams = (tx2 as SolanaTransactionRpc).params;
      expect(params2).toBeDefined();
      expect(params2.transaction).toBeDefined();
      expect(params2.transaction.length).toBeGreaterThan(0);
      expect(params2.pubkey).toBe(solanaMainnetSender);
    });
  });
});