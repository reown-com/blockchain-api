import { getTestSetup } from './init';

describe('Middlewares', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  it('OK response should contain x-request-id header', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63/history?projectId=${projectId}`,
    )
    expect(resp.headers).toBeDefined();
    expect(resp.status).toBe(200);
    // Check if the header value is a valid UUIDv4
    const uuidv4Pattern = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    expect(resp.headers['x-request-id']).toMatch(uuidv4Pattern);
  })
  it('Error response should contain x-request-id header', async () => {
    // Wrong address request
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/0Ff3ea39310011333095CFCcCc7c4Ad74034CABA63/history?projectId=${projectId}`,
    )
    expect(resp.headers).toBeDefined();
    expect(resp.status).toBe(400);
    // Check if the header value is a valid UUIDv4
    const uuidv4Pattern = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    expect(resp.headers['x-request-id']).toMatch(uuidv4Pattern);
  })
})
