import { getTestSetup } from './init';

describe('Fungible price', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const endpoint = `${baseUrl}/v1/fungible/price`;
  const currency = 'usd'
  const native_token_address = '0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee';

  it('get erc20 mainnet token price', async () => {
    const shib_token_address = 'eip155:1:0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce';
    let request_data = {
      projectId: projectId,
      currency: currency,
      addresses: [shib_token_address]
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
      expect(item.symbol).toBe('SHIB')
      expect(typeof item.iconUrl).toBe('string')
      expect(typeof item.price).toBe('number')
    }
  })

  it('get native tokens price', async () => {
    const native_tokens = [
      { chainId: 1, symbol: 'ETH' },
      { chainId: 56, symbol: 'BNB' },
      { chainId: 100, symbol: 'XDAI' },
      { chainId: 137, symbol: 'MATIC' },
      { chainId: 250, symbol: 'FTM' },
      { chainId: 43114, symbol: 'AVAX' },
    ];

    for (const token of native_tokens) {
      let request_data = {
        projectId: projectId,
        currency: currency,
        addresses: [`eip155:${token.chainId}:${native_token_address}`],
        chainId: token.chainId
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
        expect(item.symbol).toBe(token.symbol)
        expect(typeof item.iconUrl).toBe('string')
        expect(typeof item.price).toBe('number')
      }
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
      addresses: [`eip155:1:${native_token_address}`]
    }
    resp = await httpClient.post(
      `${endpoint}`,
      request_data
    )
    expect(resp.status).toBe(422)
    
  })
})
