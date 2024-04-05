import { getTestSetup } from './init';

describe('Fungible price', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const endpoint = `${baseUrl}/v1/fungible/price`;

  // RSR token address
  const implementation_address = 'eip155:1:0x320623b8e4ff03373931769a31fc52a4e78b5d70'
  const currency = 'usd'

  it('get token price', async () => {
    let request_data = {
      projectId: projectId,
      currency: currency,
      addresses: [implementation_address]
    }
    let resp: any = await httpClient.post(
      `${endpoint}`,
      request_data
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.fungibles).toBe('object')
    expect(resp.data.fungibles.length).toBeGreaterThan(0)

    for (const item of resp.data.fungibles) {
      expect(typeof item.name).toBe('string')
      expect(typeof item.symbol).toBe('string')
      expect(typeof item.iconUrl).toBe('string')
      expect(typeof item.price).toBe('number')
    }
  })

  it('bad arguments', async () => {
    // Empty addresses
    let request_data = {
      projectId: projectId,
      currency: currency,
      addresses: []
    }
    let resp: any = await httpClient.post(
      `${endpoint}`,
      request_data
    )
    expect(resp.status).toBe(400)

    // Wrong currency
    request_data = {
      projectId: projectId,
      currency: "irn",
      addresses: [implementation_address]
    }
    resp = await httpClient.post(
      `${endpoint}`,
      request_data
    )
    expect(resp.status).toBe(422)
    
  })
})
