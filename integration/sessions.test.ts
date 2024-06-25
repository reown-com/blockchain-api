import { getTestSetup } from './init';
import { ethers } from "ethers"

describe('Sessions/Permissions', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const address = `eip155:1:${ethers.Wallet.createRandom().address}`;
  // Session payload
  const payload = {
    permission: {
      permissionType: "exampleType",
      data: "exampleData",
      required: true,
      onChainValidated: false
    }
  }
  let new_pci: string;
  
  it('create new session', async () => {
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}`,
      payload
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.pci).toBe('string')
    new_pci = resp.data.pci
    expect(typeof resp.data.key).toBe('string')
    // check key is base64 encoded
    expect(Buffer.from(resp.data.key, 'base64').toString('base64')).toBe(resp.data.key)
  })

  it('list PCIs for address', async () => {
    let resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pci.length).toBe(1)
    expect(resp.data.pci[0]).toBe(new_pci)
  })

  it('get session by PCI', async () => {
    let resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}/${new_pci}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.permissionType).toBe(payload.permission.permissionType)
    expect(resp.data.data).toBe(payload.permission.data)
    expect(resp.data.required).toBe(payload.permission.required)
    expect(resp.data.onChainValidated).toBe(payload.permission.onChainValidated)
  })
})
