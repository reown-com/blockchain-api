import { getTestSetup } from './init';

describe('Generators', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  
  it('onramp Pay SDK URL', async () => {
    const expected_host = 'https://pay.coinbase.com/buy/select-asset';
    const address = '0x1234567890123456789012345678901234567890';
    const partnerUserId = 'someUserID';
    const payload = {
      partnerUserId,
      destinationWallets:[{ address }],
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/generators/onrampurl?projectId=${projectId}`,
      payload
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')
    expect(typeof resp.data.url).toBe('string')
    expect(resp.data.url).toContain(expected_host)
    expect(resp.data.url).toContain(address)
    expect(resp.data.url).toContain(partnerUserId)
  })
  it('onramp Pay SDK URL wrong payload', async () => {
    const address = '0x1234567890123456789012345678901234567890';
    const partnerUserId = 'someUserID';
    // Creating the wrong payload
    const payload = {
      partner: partnerUserId,
      someWallets:[{ address }],
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/generators/onrampurl?projectId=${projectId}`,
      payload
    )
    expect(resp.status).toBe(400)
  })
})
