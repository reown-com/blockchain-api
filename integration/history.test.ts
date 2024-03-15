import { getTestSetup } from './init';

describe('Transactions history', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const fulfilled_address = '0x63755B7B300228254FB7d16321eCD3B87f98ca2a'
  const empty_history_address = '0x5b6262592954B925B510651462b63ddEbcc22eaD'

  it('fulfilled history', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_address}/history?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.data).toBe('object')
    expect(resp.data.data).toHaveLength(50)
    expect(typeof resp.data.next).toBe('string')
    expect(resp.data.next).toHaveLength(80)
    
    for (const item of resp.data.data) {
      expect(item.id).toBeDefined()
      expect(typeof item.metadata).toBe('object')
      // expect chain to be null or caip-2 format
      if (item.metadata.chain !== null) {
        expect(item.metadata.chain).toEqual(expect.stringMatching(/^(eip155:)?\d+$/));
      } else {
        expect(item.metadata.chain).toBeNull();
      }
      expect(typeof item.metadata.application).toBe('object')
      expect(typeof item.transfers).toBe('object')
    }
  })
  it('empty history', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${empty_history_address}/history?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.data).toBe('object')
    expect(resp.data.data).toHaveLength(0)
    expect(resp.data.next).toBeNull()
  })
  it('wrong address', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/0X${fulfilled_address}/history?projectId=${projectId}`,
    )
    expect(resp.status).toBe(400)
  })
  it('onramp Coinbase provider', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_address}/history?onramp=coinbase&projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.next).toBe('string')
    expect(typeof resp.data.data).toBe('object')

    const first = resp.data.data[0]
    expect(first.id).toBeDefined()
    expect(first.metadata.sentFrom).toBe('Coinbase')
    expect(first.metadata.operationType).toBe('buy')
    expect(first.metadata.status).toEqual(expect.stringMatching(/^ONRAMP_TRANSACTION_STATUS_/));
  })
  it('onramp wrong provider', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_address}/history?onramp=some&projectId=${projectId}`,
    )
    expect(resp.status).toBe(400)
  })
})
