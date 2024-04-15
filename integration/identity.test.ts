import { getTestSetup } from './init';

describe('Identity', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const knownAddress = '0xf3ea39310011333095CFCcCc7c4Ad74034CABA63';
  const unknownAddress = '0xf3ea39310011333095CFCcCc7c4Ad74034CABA64';

  it('known ens with the cache enabled (default)', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${knownAddress}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe('cyberdrk.eth')
  })
  it('known ens with the cache disabled', async () => {
    // Allowed ProjectID
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${knownAddress}?chainId=eip155%3A1&projectId=${projectId}&useCache=false`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe('cyberdrk.eth')

    // Not allowed ProjectID to use `useCache`
    const notAllowedProjectId = 'someprojectid';
    resp = await httpClient.get(
      `${baseUrl}/v1/identity/${knownAddress}?chainId=eip155%3A1&projectId=${notAllowedProjectId}&useCache=false`,
    )
    expect(resp.status).toBe(401)
  })
  it('unknown ens', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${unknownAddress}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(null)
  })
})
