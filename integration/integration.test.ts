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
  describe('Middlewares', () => {
    it('headers should contain x-request-id', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63/history?projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(resp.header['x-request-id']).toBe('string');
      expect(resp.header['x-request-id']).toHaveLength(36); // UUIDv4
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
    it('known ens demo', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/identity/0x621D24169AeCf1da1eE8dce6aA2258F277434334?chainId=eip155%3A1&projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(resp.data.name).toBe('DerekDiscoDude.connect.id')
    })
    it('unknown ens', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/identity/0xf3ea39310011333095CFCcCc7c4Ad74034CABA64?chainId=eip155%3A1&projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(resp.data.name).toBe(null)
    })
  })
  describe('Transactions history', () => {
    it('fulfilled history', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63/history?projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data.data).toBe('object')
      expect(resp.data.data).toHaveLength(50)
      expect(typeof resp.data.next).toBe('string')
      expect(resp.data.next).toHaveLength(80)
    })
    it('empty history', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/0x739ff389c8eBd9339E69611d46Eec6212179BB67/history?projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data.data).toBe('object')
      expect(resp.data.data).toHaveLength(0)
      expect(resp.data.next).toBeNull()
    })
    it('wrong address', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/01739ff389c8eBd9339E69611d46Eec6212179BB67/history?projectId=${projectId}`,
      )
      expect(resp.status).toBe(400)
    })
  })
  describe('Portfolio', () => {
    it('finds portfolio items', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63/portfolio?projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      const first = resp.data.data[0]
      expect(first.id).toBeDefined()
      expect(first.name).toBeDefined()
      expect(first.symbol).toBeDefined()
    })
  })
})
