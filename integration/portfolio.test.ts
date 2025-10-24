import { getTestSetup } from './init';

describe('Portfolio', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  it('finds portfolio items', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/0x2aae531a81461f029cd55cb46703211c9227ba05/portfolio?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    const first = resp.data.data[0]
    expect(first.id).toBeDefined()
    expect(first.name).toBeDefined()
    expect(first.symbol).toBeDefined()
  })
})
