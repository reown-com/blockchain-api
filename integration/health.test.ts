import { getTestSetup } from './init';

describe('Health', () => {
  it('is healthy', async () => {
    const { baseUrl, projectId, httpClient } = getTestSetup();
    const resp: any = await httpClient.get(`${baseUrl}/health`)

    expect(resp.status).toBe(200)
    expect(resp.data).toContain('OK v')
    expect(resp.data).toContain('hash:')
    expect(resp.data).toContain('features:')
    expect(resp.data).toContain('uptime:')
  })
})
