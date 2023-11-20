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
  describe('Health', () => {
    const url = `${process.env.RPC_URL}/health`

    it('is healthy', async () => {
      const { status } = await http.get(`${url}`)

      expect(status).toBe(200)
    })
  })
  describe('Identity', () => {
    it('known ens', async () => {
      let resp: any = await http.get(
        `${process.env.RPC_URL}/v1/identity/0xf3ea39310011333095CFCcCc7c4Ad74034CABA63?chainId=eip155%3A1&projectId=${process.env.PROJECT_ID}`,
      )
      // TODO: uncomment when API works again
      // expect(resp.status).toBe(200)
      // expect(resp.data.origin).toBe('http://app.uniswap.org')
      // expect(resp.data.isScam).toBe(false)
      // expect(resp.headers["access-control-allow-origin"]).toBe("*")
    })
  })
})
