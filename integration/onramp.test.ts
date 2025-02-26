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

  it('get providers', async () => {
    let resp: any = await httpClient.get(
      `${onRampPath}/providers` +
      `?projectId=${projectId}`
    );
    expect(resp.status).toBe(200)
    expect(resp.data.length).toBeGreaterThan(0)
    expect(typeof resp.data[0].name).toBe('string')
    expect(typeof resp.data[0].serviceProvider).toBe('string')
    expect(typeof resp.data[0].logos).toBe('object')
  })

  it('get providers properties', async () => {
    // Check for `countries` type
    let type = 'countries'
    let resp: any = await httpClient.get(
      `${onRampPath}/providers/properties` +
      `?projectId=${projectId}` +
      `&type=${type}`
    );
    expect(resp.status).toBe(200)
    expect(resp.data.length).toBeGreaterThan(0)
    expect(typeof resp.data[0].countryCode).toBe('string')
    expect(typeof resp.data[0].name).toBe('string')
    expect(typeof resp.data[0].flagImageUrl).toBe('string')

    // Check for `crypto-currencies` type
    type = 'crypto-currencies'
    resp = await httpClient.get(
      `${onRampPath}/providers/properties` +
      `?projectId=${projectId}` +
      `&type=${type}`
    );
    expect(resp.status).toBe(200)
    expect(resp.data.length).toBeGreaterThan(0)
    expect(typeof resp.data[0].currencyCode).toBe('string')
    expect(typeof resp.data[0].name).toBe('string')
    expect(typeof resp.data[0].chainCode).toBe('string')
    expect(typeof resp.data[0].symbolImageUrl).toBe('string')

    // Check for `fiat-currencies` type
    type = 'fiat-currencies'
    resp = await httpClient.get(
      `${onRampPath}/providers/properties` +
      `?projectId=${projectId}` +
      `&type=${type}`
    );
    expect(resp.status).toBe(200)
    expect(resp.data.length).toBeGreaterThan(0)
    expect(typeof resp.data[0].currencyCode).toBe('string')
    expect(typeof resp.data[0].name).toBe('string')
    expect(typeof resp.data[0].symbolImageUrl).toBe('string')

    // Check for `payment-methods` type
    type = 'payment-methods'
    resp = await httpClient.get(
      `${onRampPath}/providers/properties` +
      `?projectId=${projectId}` +
      `&type=${type}`
    );
    expect(resp.status).toBe(200)
    expect(resp.data.length).toBeGreaterThan(0)
    expect(typeof resp.data[0].paymentMethod).toBe('string')
    expect(typeof resp.data[0].name).toBe('string')
    expect(typeof resp.data[0].paymentType).toBe('string')

    // Check for `fiat-purchases-limits` type
    type = 'fiat-purchases-limits'
    resp = await httpClient.get(
      `${onRampPath}/providers/properties` +
      `?projectId=${projectId}` +
      `&type=${type}`
    );
    expect(resp.status).toBe(200)
    expect(resp.data.length).toBeGreaterThan(0)
    expect(typeof resp.data[0].currencyCode).toBe('string')
    expect(typeof resp.data[0].defaultAmount).toBe('number')
    expect(typeof resp.data[0].minimumAmount).toBe('number')
    expect(typeof resp.data[0].maximumAmount).toBe('number')
  })
})
