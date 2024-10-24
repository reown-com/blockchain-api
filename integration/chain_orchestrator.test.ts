import { getTestSetup } from './init';
import { ethers, Interface } from "ethers"

describe('Chain abstraction orchestrator', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();  
  const erc20Interface = new Interface([
    'function transfer(address to, uint256 amount)',
  ]);

  // Address with 3 USDC on Base chain
  const from_address_with_funds = "0x2Aae531A81461F029cD55Cb46703211c9227ba05";
  const usdc_funds_on_address = 3_000_000;

  const receiver_address = "0x739ff389c8eBd9339E69611d46Eec6212179BB67";
  const chain_id_optimism = "eip155:10";
  const chain_id_base = "eip155:8453";
  const usdc_contract_optimism = "0x0b2c639c533813f4aa9d7837caf62653d097ff85";

  it('bridging available', async () => {
    // Sending USDC to Optimism, but having the USDC balance on Base chain
    const amount_to_send_in_decimals = usdc_funds_on_address - 1_000_000
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send_in_decimals,
    ]);

    let transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contract_optimism,
        value: "0x00", // Zero native tokens
        gas: "0x00",
        gasPrice: "0x00",
        data: data_encoded,
        nonce: "0x00",
        maxFeePerGas: "0x00",
        maxPriorityFeePerGas: "0x00",
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/check?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.requiresMultiChain).toBe('boolean')
    expect(resp.data.requiresMultiChain).toBe(true)
  })

  it('bridging unavailable (insufficient funds)', async () => {
    // Having the USDC balance on Base chain less then the amount to send
    const amount_to_send_in_decimals = usdc_funds_on_address + 1_000_000
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send_in_decimals,
    ]);

    let transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contract_optimism,
        value: "0x00", // Zero native tokens
        gas: "0x00",
        gasPrice: "0x00",
        data: data_encoded,
        nonce: "0x00",
        maxFeePerGas: "0x00",
        maxPriorityFeePerGas: "0x00",
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/check?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.requiresMultiChain).toBe('boolean')
    expect(resp.data.requiresMultiChain).toBe(false)
  })

  it('bridging unavailable (empty wallet)', async () => {
    // Checking an empty wallet
    const amount_to_send_in_decimals = usdc_funds_on_address
    const empty_wallet_address = ethers.Wallet.createRandom().address
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send_in_decimals,
    ]);

    let transactionObj = {
      transaction: {
        from: empty_wallet_address,
        to: usdc_contract_optimism,
        value: "0x00", // Zero native tokens
        gas: "0x00",
        gasPrice: "0x00",
        data: data_encoded,
        nonce: "0x00",
        maxFeePerGas: "0x00",
        maxPriorityFeePerGas: "0x00",
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/check?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.requiresMultiChain).toBe('boolean')
    expect(resp.data.requiresMultiChain).toBe(false)
  })

  it('bridging routes (no routes)', async () => {
    // Sending USDC to Optimism, having the USDC balance on Base chain
    // with MIN_AMOUNT_NOT_MET error expected
    const amount_to_send_in_decimals = 20_000 // Less then minimum amount required
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send_in_decimals,
    ]);

    let transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contract_optimism,
        value: "0x00", // Zero native tokens
        gas: "0x00",
        gasPrice: "0x00",
        data: data_encoded,
        nonce: "0x00",
        maxFeePerGas: "0x00",
        maxPriorityFeePerGas: "0x00",
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(400)
  })

  it('bridging routes (routes available)', async () => {
    // Sending USDC to Optimism, but having the USDC balance on Base chain
    const amount_to_send_in_decimals = usdc_funds_on_address - 1_000_000
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send_in_decimals,
    ]);

    let transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contract_optimism,
        value: "0x00", // Zero native tokens
        gas: "0x00",
        gasPrice: "0x00",
        data: data_encoded,
        nonce: "0x00",
        maxFeePerGas: "0x00",
        maxPriorityFeePerGas: "0x00",
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)

    const data = resp.data
    expect(typeof data.orchestrationId).toBe('string')
    // First transaction expected to be the bridging to the Base
    expect(data.transactions[0].chainId).toBe(chain_id_base)
    // The second transaction expected to be the initial one
    expect(data.transactions[1].chainId).toBe(chain_id_optimism)
  })
})
