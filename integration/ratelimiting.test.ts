import { getTestSetup } from './init';

describe('Rate limiting', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  it('Simulate flood and check is rate limited', async () => {
    // Using default max tokens of 100
    const max_tokens = 100;

    // Flooding requests twice then max tokens
    const requests_to_send = max_tokens * 2;
    
    // Sending flood requests to the generators endpoint since it's not dependent on the third parties
    const payload = {
      partnerUserId: 'someUserID',
      destinationWallets:[{ address: '0x1234567890123456789012345678901234567890' }],
    };
    const promises = [];
    for (let i = 0; i < requests_to_send; i++) {
      promises.push(
        httpClient.post(`${baseUrl}/v1/generators/onrampurl?projectId=${projectId}`, payload)
      );
    }
    const results = await Promise.allSettled(promises);
    
    let ok_statuses_counter = 0;
    let rate_limited_statuses_counter = 0;
    results.forEach((result) => {
      if (result.status === 'fulfilled' && result.value.status === 429) {
        rate_limited_statuses_counter++;
      }else if (result.status === 'fulfilled' && result.value.status === 200) {
        ok_statuses_counter++;
      }
    });

    console.log(`➜ Rate limited statuses: ${rate_limited_statuses_counter} out of ${requests_to_send} total requests.`);

    // Check if there are any successful and rate limited statuses
    expect(ok_statuses_counter).toBeGreaterThan(0);
    expect(rate_limited_statuses_counter).toBeGreaterThan(0);
  })

  it('Flood below max tokens and check is NOT rate limited', async () => {
    // Using default max tokens of 100
    const max_tokens = 100;
   
    // Sending flood requests to the endpoint other then in first test case
    // since the key is composite with the matched endpoint
    const promises = [];
    for (let i = 0; i < max_tokens; i++) {
      promises.push(
        httpClient.get(`${baseUrl}/health`)
      );
    }
    const results = await Promise.allSettled(promises);
    
    let ok_statuses_counter = 0;
    let rate_limited_statuses_counter = 0;
    results.forEach((result) => {
      if (result.status === 'fulfilled' && result.value.status === 429) {
        rate_limited_statuses_counter++;
      }else if (result.status === 'fulfilled' && result.value.status === 200) {
        ok_statuses_counter++;
      }
    });

    console.log(`➜ Rate limited statuses: ${rate_limited_statuses_counter} out of ${max_tokens} total requests.`);

    // Check if there are no any rate limited statuses
    expect(ok_statuses_counter).toBe(max_tokens);
    expect(rate_limited_statuses_counter).toBe(0);
  })
})
