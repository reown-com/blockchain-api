import { getTestSetup } from './init';
import { ethers } from "ethers"

describe('Account profile names', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  // Generate a new eth wallet
  const wallet = ethers.Wallet.createRandom();
  const address = wallet.address;
  const coin_type = 60; // ENSIP-11 Ethereum Mainnet
  const attributes = {
    bio: 'integration test domain',
  };

  // Generate a random name
  const randomString = Array.from({ length: 10 }, 
    () => (Math.random().toString(36)[2] || '0')).join('')
  const zone =  'reown.id';
  const name = `integration-test-${randomString}.${zone}`;

  // Create a message to sign
  const registerMessageObject = {
      name,
      attributes,
      timestamp: Math.round(Date.now() / 1000)
  };
  const registerMessage = JSON.stringify(registerMessageObject);

  it('register with wrong signature', async () => {
    // Sign the message
    const signature = await wallet.signMessage('some other message');

    const payload = {
      message: registerMessage,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(401)
  })

  it('register with wrong attributes', async () => {
    // Create a message to sign with wrong attributes
    const wrongAttributesMessageObject = {
      name,
      attributes: { someAttribute: 'some attribute name' },
      timestamp: Math.round(Date.now() / 1000)
    };
    const message = JSON.stringify(wrongAttributesMessageObject);
    const signature = await wallet.signMessage(message);

    const payload = {
      message,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)
  })

  it('register with wrong name format (wrong characters)', async () => {
    // Create a message to sign with wrong name format
    const wrongNameFormatMessageObject = {
      name: `!name.${zone}`,
      attributes: { bio: 'some attribute name' },
      timestamp: Math.round(Date.now() / 1000)
    };
    const message = JSON.stringify(wrongNameFormatMessageObject);
    const signature = await wallet.signMessage(message);

    const payload = {
      message,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)
  })

  it('register with wrong name format (length)', async () => {
    // Check for the short name (<3 characters)
    let randomString = Array.from({ length: 2 }, 
      () => (Math.random().toString(36)[2] || '0')).join('')
    const shortNameLengthMessageObject = {
      name: `${randomString}.${zone}`,
      attributes: { bio: 'some attribute name' },
      timestamp: Math.round(Date.now() / 1000)
    };
    let message = JSON.stringify(shortNameLengthMessageObject);
    let signature = await wallet.signMessage(message);

    let payload = {
      message,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)

    // Check for the long name (>64 characters)
    randomString = Array.from({ length: 65 }, 
      () => (Math.random().toString(36)[2] || '0')).join('')
    const longNameLengthMessageObject = {
      name: `${randomString}.${zone}`,
      attributes: { bio: 'some attribute name' },
      timestamp: Math.round(Date.now() / 1000)
    };
    message = JSON.stringify(longNameLengthMessageObject);
    signature = await wallet.signMessage(message);

    payload = {
      message,
      signature,
      coin_type,
      address,
    };
    resp = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)
  })

  it('register with wrong name zone (subdomain)', async () => {
    // Create a message to sign with wrong name format
    const wrongNameZoneMessageObject = {
      name: `test.${randomString}.${zone}`,
      attributes: { bio: 'some attribute name' },
      timestamp: Math.round(Date.now() / 1000)
    };
    const message = JSON.stringify(wrongNameZoneMessageObject);
    const signature = await wallet.signMessage(message);

    const payload = {
      message,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)
  })

  it('register with wrong name zone (root zone)', async () => {
    // Create a message to sign with wrong name format
    const wrongNameZoneMessageObject = {
      name: `${randomString}.connect.id`,
      attributes: { bio: 'some attribute name' },
      timestamp: Math.round(Date.now() / 1000)
    };
    const message = JSON.stringify(wrongNameZoneMessageObject);
    const signature = await wallet.signMessage(message);

    const payload = {
      message,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)
  })

  it('register new name', async () => {
    // Sign the message
    const signature = await wallet.signMessage(registerMessage);

    const payload = {
      message: registerMessage,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(name)
    expect(typeof resp.data.addresses).toBe('object')
    const mainnet_address = resp.data.addresses[coin_type]
    expect(mainnet_address.address).toBe(address)
  })

  it('register new name with not Mainnet coin type', async () => {
    // If the user registering new name with not Mainnet coin type
    // it should be added automatically to the addresses list
    const randomString = Array.from({ length: 10 }, 
      () => (Math.random().toString(36)[2] || '0')).join('')
    const name = `integration-test-${randomString}.${zone}`;
    const registerMessageObject = {
        name,
        attributes,
        timestamp: Math.round(Date.now() / 1000)
    };
    const registerMessage = JSON.stringify(registerMessageObject);
    const signature = await wallet.signMessage(registerMessage);

    const payload = {
      message: registerMessage,
      signature,
      coin_type: 2147483748, // ENSIP-11 xdai
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(name)
    expect(typeof resp.data.addresses).toBe('object')
    const mainnet_address = resp.data.addresses[60]
    expect(mainnet_address.address).toBe(address)
    const xdai_address = resp.data.addresses[2147483748]
    expect(xdai_address.address).toBe(address)
  })

  it('try register already registered name', async () => {
    // Sign the message
    const signature = await wallet.signMessage(registerMessage);

    const payload = {
      message: registerMessage,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)
  })

  it('try register already registered name with different coin_type', async () => {
    // Sign the message
    const signature = await wallet.signMessage(registerMessage);
    const coin_type = 2147483748; // ENSIP-11 xdai

    const payload = {
      message: registerMessage,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account`,
      payload
    )
    expect(resp.status).toBe(400)
  })

  it('name forward lookup (name found)', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/profile/account/${name}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(name)
    expect(resp.data.attributes['bio']).toBe(attributes['bio'])
    expect(typeof resp.data.addresses).toBe('object')
    // ENSIP-11 using the 60 for the Ethereum mainnet
    const first = resp.data.addresses[coin_type]
    expect(first.address).toBe(address)
  })

  it('name forward lookup (name not found)', async () => {
    const randomString = Array.from({ length: 10 }, 
      () => (Math.random().toString(36)[2] || '0')).join('')
    const name = `integration-test-${randomString}.${zone}`;
    
    // Test default behavior where 404 is returned
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/profile/account/${name}`
    )
    expect(resp.status).toBe(404)
    
    // Test apiVersion=2 where 200 and empty array is returned
    resp = await httpClient.get(
      `${baseUrl}/v1/profile/account/${name}?apiVersion=2`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')
    expect(resp.data.length).toBe(0)
  })

  it('name reverse lookup (name found)', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/profile/reverse/${address}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')
    const first_name = resp.data[0]
    expect(first_name.name).toBe(name)
    expect(typeof first_name.addresses).toBe('object')
    // ENSIP-11 using the 60 for the Ethereum mainnet
    const first_address = first_name.addresses[coin_type]
    expect(first_address.address).toBe(address)
  })

  it('name reverse lookup (name not found)', async () => {
    // Generate a new eth wallet that have no name registered
    const wallet = ethers.Wallet.createRandom();
    const address = wallet.address;

    // Test default behavior where 404 is returned
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/profile/reverse/${address}`
    )
    expect(resp.status).toBe(404)
    
    // Test apiVersion=2 where 200 and empty array is returned
    resp = await httpClient.get(
      `${baseUrl}/v1/profile/reverse/${address}?apiVersion=2`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')
    expect(resp.data.length).toBe(0)
  })

  it('name reverse lookup (identity endpoint)', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/identity/${address}?projectId=${projectId}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data).toBe('object')
    expect(resp.data.name).toBe(name)
  })

  it('update name attributes', async () => {
    // Prepare updated attributes payload
    const randomBioString = Array.from({ length: 24 }, 
      () => (Math.random().toString(36)[2] || '0')).join('')
    const updatedAttributes = {
      bio: randomBioString,
    };
    const updateAttributesMessageObject = {
      attributes: updatedAttributes,
      timestamp: Math.round(Date.now() / 1000)
    };
    const updateMessage = JSON.stringify(updateAttributesMessageObject);

    // Sign the message
    const signature = await wallet.signMessage(updateMessage);

    const payload = {
      message: updateMessage,
      signature,
      coin_type,
      address,
    };
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account/${name}/attributes`,
      payload
    );
    
    expect(resp.status).toBe(200)
    expect(resp.data['bio']).toBe(updatedAttributes['bio'])
  })

  it('update name address', async () => {
    // Generate a new eth wallet
    const new_address = ethers.Wallet.createRandom().address;

    // Prepare updated address payload
    const UpdateAddressMessageObject = {
      coin_type,
      address: new_address,
      timestamp: Math.round(Date.now() / 1000)
    };
    const updateMessage = JSON.stringify(UpdateAddressMessageObject);

    // Sign the message
    const signature = await wallet.signMessage(updateMessage);

    const payload = {
      message: updateMessage,
      signature,
      coin_type,
      address,
    };

    // Update the address
    let resp: any = await httpClient.post(
      `${baseUrl}/v1/profile/account/${name}/address`,
      payload
    );
    expect(resp.status).toBe(200)
    expect(resp.data[coin_type].address).toBe(new_address)

    // Query the name to see if the address was updated
    resp = await httpClient.get(
      `${baseUrl}/v1/profile/account/${name}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(name)
    expect(typeof resp.data.addresses).toBe('object')
    const first = resp.data.addresses[coin_type]
    expect(first.address).toBe(new_address)
  })

  it('name suggestions', async () => {
    const test_name_suggest = 'max';
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/profile/suggestions/${test_name_suggest}?zone=${zone}`
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.suggestions).toBe('object')
    let suggestions = resp.data.suggestions;
    // Minimum 3 suggestions should be returned
    expect(suggestions.length).toBeGreaterThan(3)
    // First suggestion should be the exact match
    expect(suggestions[0].name).toBe(`${test_name_suggest}.${zone}`)
    expect(typeof suggestions[0].registered).toBe('boolean')
  })
})
