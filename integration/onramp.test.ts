import { getTestSetup } from './init';

describe('OnRamp', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();
  const onRampPath = `${baseUrl}/v1/onramp`;
  const country = 'US';
  const subdivision = 'NY';

  it('buy options', async () => {
    let resp: any = await httpClient.get(
      `${onRampPath}/buy/options?projectId=${projectId}&country=${country}&subdivision=${subdivision}`,
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

  it('buy quotes', async () => {
    let resp: any = await httpClient.get(
      `${onRampPath}/buy/quotes` +
      `?projectId=${projectId}` +
      `&country=${country}` +
      `&subdivision=${subdivision}` +
      `&purchaseCurrency=BCH` +
      `&paymentAmount=100.00` +
      `&paymentCurrency=USD` +
      `&paymentMethod=CARD`,
    );
    expect(resp.status).toBe(200)

    const checkValueAndCurrency = (obj: any) => {
      expect(typeof obj.value).toBe('string')
      expect(typeof obj.currency).toBe('string')
    }
    
    expect(typeof resp.data.quoteId).toBe('string')
    checkValueAndCurrency(resp.data.paymentTotal)
    checkValueAndCurrency(resp.data.paymentSubTotal)
    checkValueAndCurrency(resp.data.purchaseAmount)
    checkValueAndCurrency(resp.data.coinbaseFee)
    checkValueAndCurrency(resp.data.networkFee)
  })
})
