import { getTestSetup } from './init'
import { ethers } from 'ethers'

const contractCallPermission = {
  type: 'contract-call',
  data: {
    address: '0x2E65BAfA07238666c3b239E94F32DaD3cDD6498D',
  },
}

const NativeTokenRecurringAllowancePermission = {
  type: 'native-token-recurring-allowance',
  data: {
    start: Math.floor(Date.now() / 1000),
    period: 86400,
    allowance:
      '0x00000000000000000000000000000000000000000000000000005AF3107A4000', // 0.0001
  },
}

const permissionContext = '0x00'

describe('Sessions/Permissions', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup()
  const address = `eip155:1:${ethers.Wallet.createRandom().address}`
  // New session PCI
  let new_pci: string
  // New session signing (private) key
  // let signing_key: string

  it('create new session - v1 format (default)', async () => {
    const permission = {
      expiry: Math.floor(Date.now() / 1000) + 3600,
      signer: {
        type: 'k256',
        data: '0x',
      },
      permissions: [
        contractCallPermission,
        NativeTokenRecurringAllowancePermission,
      ],
      policies: [],
    }

    const resp = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`,
      permission,
    )

    expect(resp.status).toBe(200)
    expect(typeof resp.data.pci).toBe('string')
    new_pci = resp.data.pci
    expect(typeof resp.data.key).toBe('object')
    expect(resp.data.key.type).toBe('secp256k1')
    // v1 format: ASCII-hex encoded with 0x prefix, 262 characters (260 + 2 for "0x")
    expect(resp.data.key.publicKey.startsWith('0x')).toBe(true)
    expect(resp.data.key.publicKey.length).toBe(262)
  })

  it('create new session - v2 format', async () => {
    // Use a different address to avoid interfering with main test flow
    const v2Address = `eip155:1:${ethers.Wallet.createRandom().address}`
    
    const permission = {
      expiry: Math.floor(Date.now() / 1000) + 3600,
      signer: {
        type: 'k256',
        data: '0x',
      },
      permissions: [
        contractCallPermission,
        NativeTokenRecurringAllowancePermission,
      ],
      policies: [],
    }

    const resp = await httpClient.post(
      `${baseUrl}/v1/sessions/${v2Address}?projectId=${projectId}&v=2`,
      permission,
    )

    expect(resp.status).toBe(200)
    expect(typeof resp.data.pci).toBe('string')
    expect(typeof resp.data.key).toBe('object')
    expect(resp.data.key.type).toBe('secp256k1')
    // v2 format: Direct hex with 0x prefix, 132 characters
    expect(resp.data.key.publicKey.startsWith('0x')).toBe(true)
    expect(resp.data.key.publicKey.length).toBe(132)
  })

  it('list PCIs for address', async () => {
    const resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pcis.length).toBe(1)
    const pci = resp.data.pcis[0]
    expect(pci.pci).toBe(new_pci)
    expect(pci.project.id).toBe(projectId)
  })

  it('update PCI permission context (activate)', async () => {
    const context = {
      pci: new_pci,
      expiry: Math.floor(Date.now() / 1000) + 3600,
      signer: {
        type: 'k256',
        data: '0x',
      },
      permissions: [contractCallPermission],
      policies: [],
      context: permissionContext,
    }

    const resp = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}/activate?projectId=${projectId}`,
      context,
    )

    expect(resp.status).toBe(200)
  })

  it('get session context by PCI', async () => {
    const resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}/getcontext?projectId=${projectId}&pci=${new_pci}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.context).toBe(permissionContext)
  })

  it('revoke PCI permission', async () => {
    // Check PCI is exists
    let resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pcis.length).toBe(1)
    expect(resp.data.pcis[0].pci).toBe(new_pci)
    expect(resp.data.pcis[0].revokedAt).toBe(null)

    const payload = {
      pci: new_pci,
    }

    // Revoke PCI
    resp = await httpClient.post(
      `${baseUrl}/v1/sessions/${address}/revoke?projectId=${projectId}`,
      payload,
    )
    expect(resp.status).toBe(200)

    // check PCI is revoked
    resp = await httpClient.get(
      `${baseUrl}/v1/sessions/${address}?projectId=${projectId}`,
    )
    expect(resp.status).toBe(200)
    expect(resp.data.pcis.length).toBe(1)
    // Check revokedAt is fullfilled
    expect(typeof resp.data.pcis[0].revokedAt).toBe('number')
  })
})
