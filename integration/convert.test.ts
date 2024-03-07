import { getTestSetup } from './init';

describe('Token conversion', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const namespace = 'eip155'
  const chainId = '1';
  const caip2_chain_id = `${namespace}:${chainId}`;

  it('available tokens list', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/convert/tokens?projectId=${projectId}&chainId=${caip2_chain_id}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.tokens).toBe('object')
    expect(resp.data.tokens.length).toBeGreaterThan(1)

    for (const token of resp.data.tokens) {
      expect(typeof token.name).toBe('string')
      expect(typeof token.symbol).toBe('string')
      expect(token.address).toEqual(expect.stringMatching(new RegExp(`^${caip2_chain_id}:.+$`)))
      expect(typeof token.decimals).toBe('number')
      if (token.logoUri !== null) {
        expect(token.logoUri).toEqual(expect.stringMatching(/^(https:\/\/|ipfs:\/\/).+$/));
      } else {
        expect(token.logoUri).toBeNull();
      }
      if (token.eip2612 !== null) {
        expect(typeof token.eip2612).toBe('boolean');
      }
    }
  })
})
