import { getTestSetup } from './init';
import { ethers } from "ethers"
import { canonicalize } from 'json-canonicalize';
import { createSign, createPrivateKey } from 'crypto';

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
  // New session PCI
  let new_pci: string;
  // New session signing (private) key
  let signing_key: string;
  
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
    signing_key = resp.data.key
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

  it('update PCI permission context', async () => {
    const context = {
      expiry: 1234567890,
      factory: "exampleFactory",
      factoryData: "exampleFactoryData",
      permissionsContext: "examplePermissionsContext",
      signer: {
        type: "exampleType",
        data:{
          ids: ["exampleId1", "exampleId2"]
        }, 
      },
      signerData:{
        userOpBuilder: "exampleUserOpBuilder",
      }
    }
    
    const payload = {
      pci: new_pci,
      context
    }
    
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}/context`,
      payload
    )
    expect(resp.status).toBe(200)
  })

  it('revoke PCI permission', async () => {
    // Check PCI is exists
    let resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pci.length).toBe(1)
    expect(resp.data.pci[0]).toBe(new_pci)

    let payload = {
      pci: new_pci,
    }
    
    // Revoke PCI
    resp = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}/revoke`,
      payload
    )
    expect(resp.status).toBe(200)

    // check PCI is revoked
    resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pci.length).toBe(0)
  })
})
