import { getTestSetup } from './init';

describe('Supported chains', () => {
  const { baseUrl, httpClient } = getTestSetup();

  it('Returns Ethereum Mainnet', async () => {
    const resp = await httpClient.get(`${baseUrl}/v1/supported-chains`)
    expect(resp.status).toBe(200)
    expect(resp.data).toContain('eip155:1')
  })
})
