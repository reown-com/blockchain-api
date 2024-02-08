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
  const name = `${randomString}.connect.test`;

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

  it('name forward lookup', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/profile/account/${name}`
    )
    expect(resp.status).toBe(200)
    expect(resp.data.name).toBe(name)
    expect(resp.data.attributes['bio']).toBe(attributes['bio'])
    expect(typeof resp.data.addresses).toBe('object')
    // ENSIP-11 using the 60 for the Ethereum mainnet
    const first = resp.data.addresses["60"]
    expect(first.address).toBe(address)
  })

  it('name reverse lookup', async () => {
    let resp: any = await httpClient.get(
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
})
