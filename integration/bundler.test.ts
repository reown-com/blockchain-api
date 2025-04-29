import { getTestSetup } from './init';

describe('Bundler operations', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const sepoliaChainId = 'eip155:11155111'
  const mainnetChainId = 'eip155:1'
  const method = 'eth_getUserOperationReceipt'
  const successOperationTxHash = '0x772b10c68cb2470259be889b97e87618a4d8fc2b21767503724a9842bc83b5de'

  it('unsupported method', async () => {
    let json_rpc = {
      jsonrpc: '2.0',
      method: 'eth_chainId',
      params: [successOperationTxHash],
      id: 1
    }
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/bundler?projectId=${projectId}&chainId=${sepoliaChainId}`,
      json_rpc
    )
    expect(resp.status).toBe(422)
  })

  it('no receipt', async () => {
    let json_rpc = {
      jsonrpc: '2.0',
      method,
      params: [successOperationTxHash],
      id: 1
    }
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/bundler?projectId=${projectId}&chainId=${mainnetChainId}`,
      json_rpc
    )
    expect(resp.status).toBe(200)
    expect(resp.data.result).toBeNull()
  })

  // Temporary disabling until fix the correct successOperationTxHash
  xit('successful receipt', async () => {
    let json_rpc = {
      jsonrpc: '2.0',
      method,
      params: [successOperationTxHash],
      id: 1
    }
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/bundler?projectId=${projectId}&chainId=${sepoliaChainId}`,
      json_rpc
    )
    expect(resp.status).toBe(200)
    expect(resp.data.result.success).toBe(true)
  })
})
