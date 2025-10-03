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

  it('Chain info RPC method Ton', async () => {
    const payload = {
      jsonrpc: "2.0",
      method: "getMasterchainInfo",
      params: {},
      id: 1,
    };
    const chainsResp: any = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(chainsResp.status).toBe(200)
    expect(typeof chainsResp.data).toBe('object')
    const chains = chainsResp.data.http
    expect(chains.length).toBeGreaterThan(0)
    
    for (const chain of chains) {
      if (!chain.includes('ton:')) {
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

  it('Tron broadcast transaction wrapped RPC method', async () => {
    // Tron mainnet
    const tron_mainnet_chainId = "tron:0x2b6653dc";
    // Expired transaction payload, but we can check if the method is working
    const payload = {
      jsonrpc: "2.0",
      method: "tron_broadcastTransaction",
      params:[
        "0x1",
        "true",
        "{\"contract\":[{\"parameter\":{\"value\":{\"amount\":1000,\"owner_address\":\"41608f8da72479edc7dd921e4c30bb7e7cddbe722e\",\"to_address\":\"41e9d79cc47518930bc322d9bf7cddd260a0260a8d\"},\"type_url\":\"type.googleapis.com/protocol.TransferContract\"},\"type\":\"TransferContract\"}],\"ref_block_bytes\":\"5e4b\",\"ref_block_hash\":\"47c9dc89341b300d\",\"expiration\":1591089627000,\"timestamp\":1591089567635}",
        "0a025e4b220847c9dc89341b300d40f8fed3a2a72e5a66080112620a2d747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e73666572436f6e747261637412310a1541608f8da72479edc7dd921e4c30bb7e7cddbe722e121541e9d79cc47518930bc322d9bf7cddd260a0260a8d18e8077093afd0a2a72e",
        ["0a025e4b220847c9dc8934"]
      ],
      id: 1,
    };

    let resp: any = await httpClient.post(
      `${baseUrl}/v1?chainId=${tron_mainnet_chainId}&projectId=${projectId}`,
      payload
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')
    expect(typeof resp.data.result).toBe('object')
    expect(typeof resp.data.result.txid).toBe('string')

    // Tron nile testnet
    const tron_nile_testnet_chainId = "tron:0xcd8690dc";
    resp = await httpClient.post(
      `${baseUrl}/v1?chainId=${tron_nile_testnet_chainId}&projectId=${projectId}`,
      payload
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')
    expect(typeof resp.data.result).toBe('object')
    expect(typeof resp.data.result.txid).toBe('string')
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
