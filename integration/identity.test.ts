import { getTestSetup } from './init';

describe('Identity', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  it('known ens', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63?chainId=eip155%3A1&projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe('cyberdrk.eth')
  })
  it('unknown ens', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/0xf3ea39310011333095CFCcCc7c4Ad74034CABA64?chainId=eip155%3A1&projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(null)
  })
})
