import axios from 'axios'
import { ethers } from "ethers"

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
    const fulfilled_address = '0x63755B7B300228254FB7d16321eCD3B87f98ca2a'
    const empty_history_address = '0x739ff389c8eBd9339E69611d46Eec6212179BB67'

    it('fulfilled history', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/${fulfilled_address}/history?projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data.data).toBe('object')
      expect(resp.data.data).toHaveLength(50)
      expect(typeof resp.data.next).toBe('string')
      expect(resp.data.next).toHaveLength(80)
      
      for (const item of resp.data.data) {
        expect(item.id).toBeDefined()
        expect(typeof item.metadata).toBe('object')
        // expect chain to be null or caip-2 format
        if (item.metadata.chain !== null) {
          expect(item.metadata.chain).toEqual(expect.stringMatching(/^(eip155:)?\d+$/));
        } else {
          expect(item.metadata.chain).toBeNull();
        }
        expect(typeof item.metadata.application).toBe('object')
        expect(typeof item.transfers).toBe('object')
      }
    })
    it('empty history', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/${empty_history_address}/history?projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data.data).toBe('object')
      expect(resp.data.data).toHaveLength(0)
      expect(resp.data.next).toBeNull()
    })
    it('wrong address', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/0X${fulfilled_address}/history?projectId=${projectId}`,
      )
      expect(resp.status).toBe(400)
    })
    it('onramp Coinbase provider', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/${fulfilled_address}/history?onramp=coinbase&projectId=${projectId}`,
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data.next).toBe('string')
      expect(typeof resp.data.data).toBe('object')

      const first = resp.data.data[0]
      expect(first.id).toBeDefined()
      expect(first.metadata.sentFrom).toBe('Coinbase')
      expect(first.metadata.operationType).toBe('buy')
      expect(first.metadata.status).toEqual(expect.stringMatching(/^ONRAMP_TRANSACTION_STATUS_/));
    })
    it('onramp wrong provider', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/account/${fulfilled_address}/history?onramp=some&projectId=${projectId}`,
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
  
  describe('Account profile names', () => {
    // Generate a new eth wallet
    const wallet = ethers.Wallet.createRandom();
    const address = wallet.address;
    const coin_type = 60; // SLIP-44 Ethereum
    const chain_id = 1; // Ethereum mainnet
    const attributes = {
      bio: 'integration test domain',
    };

    // Generate a random name
    const randomString = Array.from({ length: 10 }, 
      () => (Math.random().toString(36)[2] || '0')).join('')
    const name = `${randomString}.connect.test`;

    // Create a message to sign
    const messageObject = {
        name,
        coin_type,
        chain_id,
        address,
        attributes,
        timestamp: Math.round(Date.now() / 1000)
    };
    const message = JSON.stringify(messageObject);

    it('register with wrong signature', async () => {
      // Sign the message
      const signature = await wallet.signMessage('some other message');

      const payload = {
        message,
        signature,
        coin_type,
        address,
      };
      let resp: any = await http.post(
        `${baseUrl}/v1/profile/account/${name}`,
        payload
      )
      expect(resp.status).toBe(401)
    })
    it('register new name', async () => {
      // Sign the message
      const signature = await wallet.signMessage(message);

      const payload = {
        message,
        signature,
        coin_type,
        address,
      };
      let resp: any = await http.post(
        `${baseUrl}/v1/profile/account/${name}`,
        payload
      )
      expect(resp.status).toBe(200)
    })
    it('try register already registered name', async () => {
      // Sign the message
      const signature = await wallet.signMessage(message);

      const payload = {
        message,
        signature,
        coin_type,
        address,
      };
      let resp: any = await http.post(
        `${baseUrl}/v1/profile/account/${name}`,
        payload
      )
      expect(resp.status).toBe(400)
    })
    it('inconsistent payload', async () => {
      // Name in payload is different from the one in the request path
      const signature = await wallet.signMessage(message);

      const payload = {
        message,
        signature,
        coin_type,
        address,
      };
      let resp: any = await http.post(
        `${baseUrl}/v1/profile/account/someothername.connect.id`,
        payload
      )
      expect(resp.status).toBe(400)
    })
    it('name forward lookup', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/profile/account/${name}`
      )
      expect(resp.status).toBe(200)
      expect(resp.data.name).toBe(name)
      expect(typeof resp.data.addresses).toBe('object')
      // ENSIP-11 using the 60 for the Ethereum mainnet
      const first = resp.data.addresses["60"]
      expect(first.address).toBe(address)
    })
    it('name reverse lookup', async () => {
      let resp: any = await http.get(
        `${baseUrl}/v1/profile/reverse/${address}`
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data).toBe('object')
      const first_name = resp.data[0]
      expect(first_name.name).toBe(name)
      expect(typeof first_name.addresses).toBe('object')
      // ENSIP-11 using the 60 for the Ethereum mainnet
      const first_address = first_name.addresses["60"]
      expect(first_address.address).toBe(address)
    })
  })

  describe('Generators', () => {
    it('onramp Pay SDK URL', async () => {
      const expected_host = 'https://pay.coinbase.com/buy/select-asset';
      const address = '0x1234567890123456789012345678901234567890';
      const partnerUserId = 'someUserID';
      const payload = {
        partnerUserId,
        destinationWallets:[{ address }],
      };
      let resp: any = await http.post(
        `${baseUrl}/v1/generators/onrampurl?projectId=${projectId}`,
        payload
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data).toBe('object')
      expect(typeof resp.data.url).toBe('string')
      expect(resp.data.url).toContain(expected_host)
      expect(resp.data.url).toContain(address)
      expect(resp.data.url).toContain(partnerUserId)
    })
    it('onramp Pay SDK URL wrong payload', async () => {
      const address = '0x1234567890123456789012345678901234567890';
      const partnerUserId = 'someUserID';
      // Creating the wrong payload
      const payload = {
        partner: partnerUserId,
        someWallets:[{ address }],
      };
      let resp: any = await http.post(
        `${baseUrl}/v1/generators/onrampurl?projectId=${projectId}`,
        payload
      )
      expect(resp.status).toBe(400)
    })
  })

  describe('Proxy', () => {
    it('Exact provider request', async () => {
      const providerId = 'Binance';
      const chainId = "eip155:56";
      const payload = {
        jsonrpc: "2.0",
        method: "eth_chainId",
        params: [],
        id: 1,
      };
      
      // Allowed projectID
      // Only allowed projectID can make this type of request
      let resp: any = await http.post(
        `${baseUrl}/v1?chainId=${chainId}&projectId=${projectId}&providerId=${providerId}`,
        payload
      )
      expect(resp.status).toBe(200)
      expect(typeof resp.data).toBe('object')

      // Not allowed projectID for this request type
      const notAllowedProjectId = 'someprojectid';
      resp = await http.post(
        `${baseUrl}/v1?chainId=${chainId}&projectId=${notAllowedProjectId}&providerId=${providerId}`,
        payload
      )
      expect(resp.status).toBe(401)
    })
  })
})
