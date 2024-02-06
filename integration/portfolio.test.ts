import { getTestSetup } from './init';

describe('Portfolio', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  it('finds portfolio items', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63/portfolio?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    const first = resp.data.data[0]
    expect(first.id).toBeDefined()
    expect(first.name).toBeDefined()
    expect(first.symbol).toBeDefined()
  })
})
