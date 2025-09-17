import { getTestSetup } from './init';

type Exchange = {
  id: string;
  name: string;
  imageUrl?: string;
}


describe('Exchanges', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const shouldSkipBinanceTest = baseUrl.includes('localhost');
  const binanceTestFn = shouldSkipBinanceTest ? it.skip : it;

  const ethAddress = 'eip155:1:0x2aae531a81461f029cd55cb46703211c9227ba05';
  const baseAddress = 'eip155:8453:0x2aae531a81461f029cd55cb46703211c9227ba05';
  const solanaAddress = 'solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:5PUrktzVvJPNFYpxNzFkGp4a5Dcj1Dduif5dAzuUUhsr';
  
  const ethUSDC = 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48';
  const baseUSDC = 'eip155:8453/erc20:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913';
  const nativeETH = 'eip155:1/slip44:60';
  const nativeSOL = 'solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp/slip44:501';
  const unsupportedAsset = 'eip155:999/erc20:0x1234567890123456789012345678901234567890';

  const supportedExchanges = ['binance', 'coinbase', 'reown_test'];

  const defaultAmount = '100';
  const hexAmount = '0x64';
  const floatAmount = '100.5';

  describe('Get Exchanges', () => {
    it('should get all exchanges without asset filter', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchanges',
        params: {
          page: 1
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(response.data.result.total).toBeGreaterThan(0);
      expect(response.data.result.exchanges).toBeInstanceOf(Array);
      expect(response.data.result.exchanges.length).toBeGreaterThan(0);

      for (const exchange of response.data.result.exchanges) {
        expect(typeof exchange.id).toBe('string');
        expect(typeof exchange.name).toBe('string');
        expect(['string', 'undefined']).toContain(typeof exchange.imageUrl);
        
        expect(supportedExchanges).toContain(exchange.id);
      }
    });

    it('should get exchanges filtered by supported asset (USDC on Base)', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchanges',
        params: {
          page: 1,
          asset: baseUSDC
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(response.data.result.total).toBeGreaterThan(0);
      expect(response.data.result.exchanges).toBeInstanceOf(Array);

      const exchangeIds = response.data.result.exchanges.map((e: Exchange) => e.id);
      expect(exchangeIds).toContain('binance');
      expect(exchangeIds).toContain('coinbase');
    });

    it('should get exchanges filtered by unsupported asset', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchanges',
        params: {
          page: 1,
          asset: unsupportedAsset
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(response.data.result.total).toBe(0);
      expect(response.data.result.exchanges).toEqual([]);
    });

    it('should filter exchanges by includeOnly parameter', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchanges',
        params: {
          page: 1,
          includeOnly: ['binance']
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(response.data.result.exchanges).toHaveLength(1);
      expect(response.data.result.exchanges[0].id).toBe('binance');
    });

    it('should filter exchanges by exclude parameter', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchanges',
        params: {
          page: 1,
          exclude: ['binance']
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      
      const exchangeIds = response.data.result.exchanges.map((e: Exchange) => e.id);
      expect(exchangeIds).not.toContain('binance');
      expect(exchangeIds).toContain('coinbase');
    });

    it('should return validation error for mutually exclusive includeOnly and exclude', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchanges',
        params: {
          page: 1,
          includeOnly: ['binance'],
          exclude: ['coinbase']
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('includeOnly and exclude are mutually exclusive');
    });
  });

  describe('Get Exchange URL', () => {
    
    binanceTestFn('should generate pay URL for Binance with USDC on Base', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'binance',
          asset: baseUSDC,
          amount: defaultAmount,
          recipient: baseAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(typeof response.data.result.url).toBe('string');
      expect(response.data.result.url).toMatch(/^https?:\/\//);
      expect(typeof response.data.result.sessionId).toBe('string');
      expect(response.data.result.sessionId.length).toBeGreaterThan(0);
    });

    it('should generate pay URL for Coinbase with USDC on Base', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'coinbase',
          asset: baseUSDC,
          amount: defaultAmount,
          recipient: baseAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(typeof response.data.result.url).toBe('string');
      expect(response.data.result.url).toMatch(/^https?:\/\//);
      expect(typeof response.data.result.sessionId).toBe('string');
      expect(response.data.result.sessionId.length).toBeGreaterThan(0);
    });

    it('should handle decimal amount format', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'coinbase',
          asset: baseUSDC,
          amount: floatAmount,
          recipient: baseAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(typeof response.data.result.url).toBe('string');
      expect(typeof response.data.result.sessionId).toBe('string');
    });

    it('should handle hexadecimal amount format', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'coinbase',
          asset: baseUSDC,
          amount: hexAmount,
          recipient: baseAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(typeof response.data.result.url).toBe('string');
      expect(typeof response.data.result.sessionId).toBe('string');
    });

    it('should generate pay URL for native ETH', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'coinbase',
          asset: nativeETH,
          amount: defaultAmount,
          recipient: ethAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(200);
      expect(response.data.result).toBeDefined();
      expect(typeof response.data.result.url).toBe('string');
      expect(typeof response.data.result.sessionId).toBe('string');
    });

    it('should return error for unknown exchange', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'unknown-exchange',
          asset: baseUSDC,
          amount: defaultAmount,
          recipient: baseAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('Exchange unknown-exchange not found');
    });

    it('should return error for unsupported asset', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'binance',
          asset: unsupportedAsset,
          amount: defaultAmount,
          recipient: 'eip155:999:0x2aae531a81461f029cd55cb46703211c9227ba05'
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('not supported');
    });

    it('should return error for mismatched recipient and asset chain', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'binance',
          asset: baseUSDC,
          amount: defaultAmount,
          recipient: ethAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('chainId must match');
    });

    it('should return error for invalid amount format', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'binance',
          asset: baseUSDC,
          amount: 'invalid-amount',
          recipient: baseAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('Invalid amount');
    });
  });

  describe('Get Exchange Buy Status', () => {
    let sessionId: string;

    beforeAll(async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'binance',
          asset: baseUSDC,
          amount: defaultAmount,
          recipient: baseAddress
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      if (response.status === 200 && response.data.result) {
        sessionId = response.data.result.sessionId;
      } else {
        sessionId = 'test-session-id-12345';
      }
    });

     
  
    it('should return error for unknown exchange', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangeBuyStatus',
        params: {
          exchangeId: 'unknown-exchange',
          sessionId: sessionId
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('Exchange unknown-exchange not found');
    });

    it('should return error for empty session ID', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangeBuyStatus',
        params: {
          exchangeId: 'binance',
          sessionId: ''
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('Invalid session ID');
    });

    it('should return error for too long session ID', async () => {
      const longSessionId = 'a'.repeat(51);

      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangeBuyStatus',
        params: {
          exchangeId: 'binance',
          sessionId: longSessionId
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('Invalid session ID');
    });
  });

  describe('Edge Cases and Error Handling', () => {
    it('should handle missing required parameters', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'reown_getExchangePayUrl',
        params: {
          exchangeId: 'binance',
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
    });

    it('should handle invalid JSON-RPC format', async () => {
      const payload = {
        id: 1,
        method: 'reown_getExchanges',
        params: {
          page: 1
        }
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(422);
    });

    it('should handle invalid method name', async () => {
      const payload = {
        jsonrpc: '2.0',
        id: 1,
        method: 'invalid_method_name',
        params: {}
      };

      const response = await httpClient.post(
        `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
        payload
      );

      expect(response.status).toBe(400);
      expect(response.data.error).toBeDefined();
      expect(response.data.error.message).toContain('Method not found');
    });
  });

  describe('Multiple Assets Support', () => {
    const testCases = [
      {
        name: 'Native ETH on Ethereum',
        asset: nativeETH,
        recipient: ethAddress,
        supportedExchanges: ['binance', 'coinbase']
      },
      // {
      //   name: 'USDC on Base',
      //   asset: baseUSDC,
      //   recipient: baseAddress,
      //   supportedExchanges: ['binance', 'coinbase']
      // },
      {
        name: 'USDC on Ethereum',
        asset: ethUSDC,
        recipient: ethAddress,
        supportedExchanges: ['binance', 'coinbase']
      },
      {
        name: 'Native SOL on Solana',
        asset: nativeSOL,
        recipient: solanaAddress,
        supportedExchanges: ['binance']
      }
    ];

    testCases.forEach(testCase => {
      const shouldSkip = baseUrl.includes('localhost') && 
                        testCase.supportedExchanges.includes('binance') &&
                        testCase.supportedExchanges.length === 1;
      
      const testFn = shouldSkip ? it.skip : it;
      
      testFn(`should support ${testCase.name}`, async () => {
        const exchangesPayload = {
          jsonrpc: '2.0',
          id: 1,
          method: 'reown_getExchanges',
          params: {
            page: 1,
            asset: testCase.asset
          }
        };

        const exchangesResponse = await httpClient.post(
          `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
          exchangesPayload
        );

        expect(exchangesResponse.status).toBe(200);
        const supportedExchangeIds = exchangesResponse.data.result.exchanges.map((e: Exchange) => e.id);
        
        testCase.supportedExchanges.forEach(expectedExchange => {
          expect(supportedExchangeIds).toContain(expectedExchange);
        });

        for (const exchangeId of testCase.supportedExchanges) {
          // Skip binance tests when running locally (for test cases with multiple exchanges)
          if (baseUrl.includes('localhost') && exchangeId === 'binance') {
            continue;
          }

          const urlPayload = {
            jsonrpc: '2.0',
            id: 1,
            method: 'reown_getExchangePayUrl',
            params: {
              exchangeId,
              asset: testCase.asset,
              amount: defaultAmount,
              recipient: testCase.recipient
            }
          };

          const urlResponse = await httpClient.post(
            `${baseUrl}/v1/json-rpc?projectId=${projectId}`,
            urlPayload
          );

          expect(urlResponse.status).toBe(200);
          expect(typeof urlResponse.data.result.url).toBe('string');
          expect(typeof urlResponse.data.result.sessionId).toBe('string');
        }
      });
    });
  });
}); 