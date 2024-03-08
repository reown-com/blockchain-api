import { getTestSetup } from './init';

describe('Token conversion (single chain)', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const namespace = 'eip155'
  const chainId = '1';
  const caip2_chain_id = `${namespace}:${chainId}`;

  const srcAsset = `${namespace}:${chainId}:0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee`;
  const destAsset = `${namespace}:${chainId}:0x111111111117dc0aa78b770fa6a738034120c302`;
  const amount = 10000000000000000;

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

  it('get conversion quote', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/convert/quotes?projectId=${projectId}&amount=${amount}&from=${srcAsset}&to=${destAsset}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.quotes).toBe('object')
    expect(resp.data.quotes.length).toBeGreaterThan(0)

    for (const quote of resp.data.quotes) {
      expect(quote.fromAmount).toEqual(`${amount}`);
      expect(quote.toAmount).toEqual(expect.stringMatching(/[0-9].*/));
    }
  })
})
