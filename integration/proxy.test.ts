import { getTestSetup } from './init';

describe('Proxy', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

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
})
