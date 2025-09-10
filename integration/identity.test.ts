import { ethers } from 'ethers';
import { getTestSetup } from './init';

describe('Identity', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const knownAddress = '0xf3ea39310011333095CFCcCc7c4Ad74034CABA63';
  const unknownAddress = '0xf3ea39310011333095CFCcCc7c4Ad74034CABA64';
  const invalidAddress = '0x1234567890123456789012345678901234567890A';
  const solanaAddress = 'So11111111111111111111111111111111111111112';

  it('known ens with the cache enabled (default)', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${knownAddress}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.headers['cache-control']).toContain('public, max-age=')
    expect(resp.data.name).toBe('cyberdrk.eth')
  })
  it('known ens with the cache disabled', async () => {
    // Allowed ProjectID
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${knownAddress}?chainId=eip155%3A1&projectId=${projectId}&useCache=false`,
    )
    expect(resp.status).toBe(200)
    expect(resp.headers['cache-control']).toContain('public, max-age=')
    const age = +resp.headers['cache-control'].match(/max-age=(\d+)/)[1];
    expect(age).toBeGreaterThan(86397)
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
    expect(resp.headers['cache-control']).toContain('public, max-age=')
    expect(resp.data.name).toBe(null)
  })
  it('random address', async () => {
    const address = ethers.Wallet.createRandom().address;
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${address}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.headers['cache-control']).toContain('public, max-age=')
    const age1 = +resp.headers['cache-control'].match(/max-age=(\d+)/)[1];
    expect(age1).toBeGreaterThan(86397)
    expect(resp.data.name).toBe(null)

    await new Promise(resolve => setTimeout(resolve, 2000));
    let resp2: any = await httpClient.get(
      `${baseUrl}/v1/identity/${address}?projectId=${projectId}`,
    )
    expect(resp2.status).toBe(200)
    const age2 = +resp2.headers['cache-control'].match(/max-age=(\d+)/)[1];
    expect(age2).toBeLessThan(86400)
    expect(age2).toBeGreaterThan(86395)
    expect(resp2.data.name).toBe(null)
  })
  it('solana address', async () => {
    const address = solanaAddress;
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${address}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(null)
  })
  it('invalid address', async () => {
    const address = invalidAddress;
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${address}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(400)
  })
})
