import axios from 'axios'

const http = axios.create({
  validateStatus: (_status) => true,
})

declare let process: {
  env: {
    RPC_URL: string
    PROJECT_ID: string
  }
}

describe('blockchain api', () => {
  let baseUrl: string
  let projectId: string
  beforeAll(() => {
    baseUrl = process.env.RPC_URL
    if (!baseUrl) {
      throw new Error('RPC_URL environment variable not set')
    }
    projectId = process.env.PROJECT_ID
    if (!projectId) {
      throw new Error('PROJECT_ID environment variable not set')
    }
  })
  describe('Health', () => {
    it('is healthy', async () => {
      const { status } = await http.get(`${baseUrl}/health`)

      expect(status).toBe(200)
    })
  })
  describe('Identity', () => {
    it('known ens', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/identity/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63?chainId=eip155%3A1&projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(resp.data.name).toBe('cyberdrk.eth')
    })
    it('unknown ens', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/identity/0xf3ea39310011333095CFCcCc7c4Ad74034CABA64?chainId=eip155%3A1&projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(resp.data.name).toBe(null)
    })
  })
})
