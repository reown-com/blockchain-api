import { getTestSetup } from './init';

describe('Fungible price', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const endpoint = `${baseUrl}/v1/fungible/price`;

  // BNB token address
  const bnb_implementation_address = 'eip155:1:0xb8c77482e45f1f44de1745f52c74426c631bdd52'
  // ETH token address representing native ETH
  const eth_native_address = 'eip155:1:0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
  const currency = 'usd'

  it('get BNB token price', async () => {
    let request_data = {
      projectId: projectId,
      currency: currency,
      addresses: [bnb_implementation_address]
    }
    let resp: any = await httpClient.post(
      `${endpoint}`,
      request_data
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.fungibles).toBe('object')
    expect(resp.data.fungibles.length).toBe(1)

    for (const item of resp.data.fungibles) {
      expect(typeof item.name).toBe('string')
      expect(item.symbol).toBe('BNB')
      expect(typeof item.iconUrl).toBe('string')
      expect(typeof item.price).toBe('number')
    }
  })

  it('get ETH native token price', async () => {
    let request_data = {
      projectId: projectId,
      currency: currency,
      addresses: [eth_native_address]
    }
    let resp: any = await httpClient.post(
      `${endpoint}`,
      request_data
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.fungibles).toBe('object')
    expect(resp.data.fungibles.length).toBe(1)

    for (const item of resp.data.fungibles) {
      expect(typeof item.name).toBe('string')
      expect(item.symbol).toBe('ETH')
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
      addresses: [eth_native_address]
    }
    resp = await httpClient.post(
      `${endpoint}`,
      request_data
    )
    expect(resp.status).toBe(422)
    
  })
})
