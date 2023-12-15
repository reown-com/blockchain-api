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
      const resp: any = await http.get(`${baseUrl}/health`)

      expect(resp.status).toBe(200)
      expect(resp.data).toContain('OK v')
      expect(resp.data).toContain('hash:')
      expect(resp.data).toContain('features:')
      expect(resp.data).toContain('uptime:')
    })
  })
  describe('Middlewares', () => {
    it('OK response should contain x-request-id header', async () => {
      let resp: any = await http.get(
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
      let resp: any = await http.get(
        `${baseUrl}/v1/account/0Ff3ea39310011333095CFCcCc7c4Ad74034CABA63/history?projectId=${projectId}`,
      )
      expect(resp.headers).toBeDefined();
      expect(resp.status).toBe(400);
      // Check if the header value is a valid UUIDv4
      const uuidv4Pattern = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
      expect(resp.headers['x-request-id']).toMatch(uuidv4Pattern);
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
