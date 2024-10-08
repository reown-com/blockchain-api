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
    const permission = {
      expiry: Math.floor(Date.now() / 1000) + 3600,
      signer: {
          type: "k256",
          data: "0x"
        },
      permissions: [
        {
          type: "custom",
          data: "0x"
        }
      ],
      policies: [
        {
          type: "custom",
          data: "0x"
        }
      ]
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`,
      permission
    )

    expect(resp.status).toBe(200)
    expect(typeof resp.data.pci).toBe('string')
    new_pci = resp.data.pci
    expect(typeof resp.data.key).toBe('object')
    expect(resp.data.key.type).toBe('secp256k1')
    // check key is string starting from 0x
    expect(resp.data.key.publicKey.startsWith('0x')).toBe(true)
  })

  it('list PCIs for address', async () => {
    let resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pcis.length).toBe(1)
    let pci = resp.data.pcis[0]
    expect(pci.pci).toBe(new_pci)
    expect(pci.project.id).toBe(projectId)
  })

  it('update PCI permission context (activate)', async () => {
    const context = {
      pci: new_pci,
      expiry: Math.floor(Date.now() / 1000) + 3600,
      signer: {
        type: "k256",
        data: "0x"
      },
      permissions: [
        {
          type: "custom",
          data: "0x"
        }
      ],
      policies: [
        {
          type: "custom",
          data: "0x"
        }
      ],
      context: "0x00"
    }
    
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}/activate?projectId=${projectId}`,
      context
    )

    expect(resp.status).toBe(200)
  })

  it('get session context by PCI', async () => {
    let resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}/${new_pci}?projectId=${projectId}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.context).toBe('0x00')
  })

  it('revoke PCI permission', async () => {
    // Check PCI is exists
    let resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pcis.length).toBe(1)
    expect(resp.data.pcis[0].pci).toBe(new_pci)

    let payload = {
      pci: new_pci,
    }
    
    // Revoke PCI
    resp = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}/revoke?projectId=${projectId}`,
      payload
    )
    expect(resp.status).toBe(200)

    // check PCI is revoked
    resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pcis.length).toBe(0)
  })
})
