import { getTestSetup } from './init';

describe('OnRamp', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const onRampPath = `${baseUrl}/v1/onramp`;

  it('buy options', async () => {
    let resp: any = await httpClient.get(
      `${onRampPath}/buy/options?projectId=${projectId}&country=US&subdivision=NY`,
    )
    expect(resp.status).toBe(200)
    
    const firstPayment = resp.data.paymentCurrencies[0]
    expect(typeof firstPayment.id).toBe('string')
    const firstPaymentLimits = firstPayment.limits[0]
    expect(typeof firstPaymentLimits.id).toBe('string')
    expect(typeof firstPaymentLimits.min).toBe('string')
    expect(typeof firstPaymentLimits.max).toBe('string')

    const firstPurchase = resp.data.purchaseCurrencies[0]
    expect(typeof firstPurchase.id).toBe('string')
    expect(typeof firstPurchase.name).toBe('string')
    expect(typeof firstPurchase.symbol).toBe('string')
    const firstPurchaseNetworks = firstPurchase.networks[0]
    expect(typeof firstPurchaseNetworks.name).toBe('string')
    expect(typeof firstPurchaseNetworks.displayName).toBe('string')
    expect(typeof firstPurchaseNetworks.contractAddress).toBe('string')
    expect(typeof firstPurchaseNetworks.chainId).toBe('string')
  })
})
