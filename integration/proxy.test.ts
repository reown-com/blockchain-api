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
})
