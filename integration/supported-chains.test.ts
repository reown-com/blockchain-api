import { getTestSetup } from './init';

describe('Supported chains', () => {
  const { baseUrl, httpClient } = getTestSetup();

  it('Returns Ethereum Mainnet', async () => {
    const resp = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(resp.status).toBe(200)
    expect(resp.data.http).toContain('eip155:1')
    expect(resp.data.http).toContain('eip155:8453')
    expect(resp.data.ws).toContain('eip155:1')
    expect(resp.data.ws).not.toContain('eip155:8453')
  })
})
