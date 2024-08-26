import { getTestSetup } from './init';

describe('Transactions history', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const fulfilled_eth_address = '0x63755B7B300228254FB7d16321eCD3B87f98ca2a'
  const fulfilled_solana_address = 'D8cjxcb8pC2SBhWesQ7oxtCRPjw4856CcvXdWzPHNCqU'

  const empty_eth_address = '0x5b6262592954B925B510651462b63ddEbcc22eaD'
  const empty_solana_address = '7ar3r6Mau1Bk7pGLWHCMj1C1bk2eCDwGWTP77j9MXTtd'

  it('fulfilled history Ethereum address', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_eth_address}/history?projectId=${projectId}`,
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

  it('fulfilled history Solana address', async () => {
    let chainId = 'solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp'
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_solana_address}/history?projectId=${projectId}&chainId=${chainId}`,
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.data).toBe('object')
    expect(resp.data.data.length).toBeGreaterThanOrEqual(2)
    
    for (const item of resp.data.data) {
      expect(item.id).toBeDefined()
      expect(typeof item.metadata).toBe('object')
      // expect chain to be null or caip-2 format
      if (item.metadata.chain !== null) {
        expect(item.metadata.chain).toEqual(expect.stringMatching(/^(solana:)?[a-zA-Z0-9]+$/));
      } else {
        expect(item.metadata.chain).toBeNull();
      }
      expect(typeof item.transfers).toBe('object')
    }
  })

  it('empty history Ethereum address', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${empty_eth_address}/history?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.data).toBe('object')
    expect(resp.data.data).toHaveLength(0)
    expect(resp.data.next).toBeNull()
  })

  it('empty history Solana address', async () => {
    let chainId = 'solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp'
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${empty_solana_address}/history?projectId=${projectId}&chainId=${chainId}`,
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.data).toBe('object')
    expect(resp.data.data).toHaveLength(0)
    expect(resp.data.next).toBeNull()
  })

  it('wrong addresses', async () => {
    // wrong Ethereum address format
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/X${fulfilled_eth_address}/history?projectId=${projectId}`,
    )
    expect(resp.status).toBe(400)

    // wrong Solana address format
    let solana_chainId = 'solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp'
    resp = await httpClient.get(
      `${baseUrl}/v1/account/X${fulfilled_solana_address}/history?projectId=${projectId}&chainId=${solana_chainId}`,
    )
    expect(resp.status).toBe(400)
  })

  it('onramp Coinbase provider', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_eth_address}/history?onramp=coinbase&projectId=${projectId}`,
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
      `${baseUrl}/v1/account/${fulfilled_eth_address}/history?onramp=some&projectId=${projectId}`,
    )
    expect(resp.status).toBe(400)
  })
})
