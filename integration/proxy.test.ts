import { getTestSetup } from './init';

describe('Proxy', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const test_evm_address = '0x2aae531a81461f029cd55cb46703211c9227ba05';
  const test_solana_address = '83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri';
  
  it('Exact provider request', async () => {
    const providerId = 'Binance';
    const chainId = "eip155:56";
    const payload = {
      jsonrpc: "2.0",
      method: "eth_chainId",
      params: [],
      id: 1,
    };
    
    // Allowed projectID
    // Only allowed projectID can make this type of request
    let resp: any = await httpClient.post(
      `${baseUrl}/v1?chainId=${chainId}&projectId=${projectId}&providerId=${providerId}`,
      payload
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')

    // Not allowed projectID for this request type
    const notAllowedProjectId = 'someprojectid';
    resp = await httpClient.post(
      `${baseUrl}/v1?chainId=${chainId}&projectId=${notAllowedProjectId}&providerId=${providerId}`,
      payload
    )
    expect(resp.status).toBe(401)
  })

  it('Balance RPC method Ethereum', async () => {
    const payload = {
      jsonrpc: "2.0",
      method: "eth_getBalance",
      params: [test_evm_address, "latest"],
      id: 1,
    };
    
    // Get the chains list from the /supported-chains endpoint
    const chainsResp: any = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(chainsResp.status).toBe(200)
    expect(typeof chainsResp.data).toBe('object')
    const chains = chainsResp.data.http
    expect(chains.length).toBeGreaterThan(0)
    
    // Check each supported eip155 chain for the eth_getBalance RPC method
    for (const chain of chains) {
      if (!chain.includes('eip155:')) {
        continue
      }
      const resp: any = await httpClient.post(
        `${baseUrl}/v1?chainId=${chain}&projectId=${projectId}`,
        payload
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data).toBe('object')
      expect(resp.data.result).toBeDefined()
      expect(resp.data.result).not.toBeNull()
      // Expect the result to be a `0x...` string
      expect(resp.data.result).toMatch(/^0x[0-9a-fA-F]+$/)
    }
  })

  it('Balance RPC method Solana', async () => {
    const payload = {
      jsonrpc: "2.0",
      method: "getBalance",
      params: [test_solana_address],
      id: 1,
    };
    const chainsResp: any = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(chainsResp.status).toBe(200)
    expect(typeof chainsResp.data).toBe('object')
    const chains = chainsResp.data.http
    expect(chains.length).toBeGreaterThan(0)
    
    for (const chain of chains) {
      if (!chain.includes('solana:')) {
        continue
      }
      const resp: any = await httpClient.post(
        `${baseUrl}/v1?chainId=${chain}&projectId=${projectId}`,
        payload
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data.result).toBe('object')
      expect(resp.data.result.value).toBeDefined()
      expect(resp.data.result.value).not.toBeNull()
      expect(resp.data.result.value).toBeGreaterThan(0)
    }
  })

  it('Block number RPC method Tron', async () => {
    const payload = {
      jsonrpc: "2.0",
      method: "eth_blockNumber",
      params: [],
      id: 1,
    };
    const chainsResp: any = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(chainsResp.status).toBe(200)
    expect(typeof chainsResp.data).toBe('object')
    const chains = chainsResp.data.http
    expect(chains.length).toBeGreaterThan(0)
    
    for (const chain of chains) {
      if (!chain.includes('tron:')) {
        continue
      }
      const resp: any = await httpClient.post(
        `${baseUrl}/v1?chainId=${chain}&projectId=${projectId}`,
        payload
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data).toBe('object')
      expect(resp.data.result).toBeDefined()
      expect(resp.data.result).not.toBeNull()
    }
  })

  it('net_listening RPC method', async () => {
    // Get the chains list from the /supported-chains endpoint
    const chainsResp: any = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(chainsResp.status).toBe(200)
    expect(typeof chainsResp.data).toBe('object')
    const chains = chainsResp.data.http
    expect(chains.length).toBeGreaterThan(0)
  
    // Check each supported eip155 chain for the net_listening RPC method
    for (const chain of chains) {
      if (!chain.includes('eip155:')) {
        continue
      }
      const payload = {
        jsonrpc: "2.0",
        method: "net_listening",
        params: [],
        id: 1,
      };

      let resp: any = await httpClient.post(
        `${baseUrl}/v1?chainId=${chain}&projectId=${projectId}`,
        payload
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data).toBe('object')
      expect(resp.data.result).toBe(true)
    }
  })

  it('net_version RPC method', async () => {
    // Get the chains list from the /supported-chains endpoint
    const chainsResp: any = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(chainsResp.status).toBe(200)
    expect(typeof chainsResp.data).toBe('object')
    const chains = chainsResp.data.http
    expect(chains.length).toBeGreaterThan(0)
  
    // Check each supported eip155 chain for the net_version RPC method
    for (const chain of chains) {
      if (!chain.includes('eip155:')) {
        continue
      }
      const chainId = chain.split(':')[1]
      const payload = {
        jsonrpc: "2.0",
        method: "net_version",
        params: [],
        id: 1,
      };

      let resp: any = await httpClient.post(
        `${baseUrl}/v1?chainId=${chain}&projectId=${projectId}`,
        payload
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data).toBe('object')
      expect(resp.data.result).toBe(chainId)
    }
  })
})
