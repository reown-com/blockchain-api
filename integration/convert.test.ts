import { getTestSetup } from './init';

describe('Token conversion (single chain)', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const namespace = 'eip155'
  const chainId = '1';
  const caip2_chain_id = `${namespace}:${chainId}`;

  const srcAsset = `${namespace}:${chainId}:0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee`;
  const destAsset = `${namespace}:${chainId}:0x111111111117dc0aa78b770fa6a738034120c302`;
  const userAddress = `${namespace}:${chainId}:0xf3ea39310011333095cfcccc7c4ad74034caba63`;
  const amount = "100000";

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

  it('unsupported chain', async () => {
    const unsupportedChainId = 'eip155:92374624';
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/convert/tokens?projectId=${projectId}&chainId=${unsupportedChainId}`
    )
    expect(resp.status).toBe(400)
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

  it('build approve tx', async () => {
    // Amount specified
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/convert/build-approve?projectId=${projectId}&amount=${amount}&from=${srcAsset}&to=${destAsset}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.tx).toBe('object')

    let tx = resp.data.tx;
    expect(tx.from).toEqual(srcAsset);
    expect(tx.to).toEqual(destAsset);
    expect(tx.data).toEqual(expect.stringMatching(/^0x.*/));
    expect(tx.eip155.gasPrice).toEqual(expect.stringMatching(/[0-9].*/));

    // Infinite amount
    resp = await httpClient.get(
      `${baseUrl}/v1/convert/build-approve?projectId=${projectId}&from=${srcAsset}&to=${destAsset}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.tx).toBe('object')

    tx = resp.data.tx;
    expect(tx.from).toEqual(srcAsset);
    expect(tx.to).toEqual(destAsset);
    expect(tx.data).toEqual(expect.stringMatching(/^0x.*/));
    expect(tx.eip155.gasPrice).toEqual(expect.stringMatching(/[0-9].*/));
  })

  it('build conversion tx', async () => {
    const payload = {
      projectId,
      amount,
      from: srcAsset,
      to: destAsset,
      userAddress,
      eip155: {
        slippage: 1,
      }
    };

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/convert/build-transaction`,
      payload
    )
    
    expect(resp.status).toBe(200)
    expect(typeof resp.data.tx).toBe('object')

    const tx = resp.data.tx;
    expect(tx.from).toEqual(userAddress);
    expect(tx.to).toEqual(expect.stringMatching(new RegExp(`^${namespace}:${chainId}:0x.*$`)));
    expect(tx.data).toEqual(expect.stringMatching(/^0x.*/));
    expect(tx.amount).toEqual(expect.stringMatching(/[0-9].*/));

    const eip155 = resp.data.tx.eip155;
    expect(eip155.gas).toEqual(expect.stringMatching(/[0-9].*/));
    expect(eip155.gasPrice).toEqual(expect.stringMatching(/[0-9].*/));
  })

  it('handling invalid parameter', async () => {
    const payload = {
      projectId,
      amount: "100000",
      from: srcAsset,
      to: destAsset,
      userAddress: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
      eip155: {
        slippage: 1,
      }
    };

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/convert/build-transaction`,
      payload
    )
    
    expect(resp.status).toBe(400)
    expect(typeof resp.data.reasons).toBe('object')
    expect(typeof resp.data.reasons[0].description).toBe('string')
  })

  it('get gas price', async () => {
    // Check for the Ethereum mainnet chain
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/convert/gas-price?projectId=${projectId}&chainId=${caip2_chain_id}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.standard).toBe('string')
    expect(typeof resp.data.fast).toBe('string')
    expect(typeof resp.data.instant).toBe('string')

    // Check for the BSC Chain as it's not support EIP1559
    resp = await httpClient.get(
      `${baseUrl}/v1/convert/gas-price?projectId=${projectId}&chainId=eip155:56`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.standard).toBe('string')
    expect(typeof resp.data.fast).toBe('string')
    expect(typeof resp.data.instant).toBe('string')

  })

  it('get allowance', async () => {
    const tokenAddress = '0x111111111117dc0aa78b770fa6a738034120c302';
    const tokenCaip10 = `${namespace}:${chainId}:${tokenAddress}`;
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/convert/allowance?projectId=${projectId}&tokenAddress=${tokenCaip10}&userAddress=${userAddress}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.allowance).toBe('string')

  })
})
