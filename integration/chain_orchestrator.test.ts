import { getTestSetup } from './init';
import { ethers, Interface } from "ethers"

describe('Chain abstraction orchestrator', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const erc20Interface = new Interface([
    'function transfer(address to, uint256 amount)',
    'function approve(address spender, uint256 amount) public returns (bool)'
  ]);

  // Address with 3 USDC on Base chain
  const from_address_with_funds = "0x2aae531a81461f029cd55cb46703211c9227ba05";
  const usdc_funds_on_base = 3_000_000;
  const usdc_funds_on_optimism = 1_057_151;
  // Amount to send to Optimism
  const amount_to_send = 3_000_000
  // Amount bridging multiplier
  const amount_multiplier = 5; // +5% topup
  // How much needs to be topped up
  const amount_to_topup = (amount_to_send - usdc_funds_on_optimism) * (100 + amount_multiplier) / 100;
  // Default gas esimation is default with 4x increase
  const gas_estimate = "0xa69ac";

  const receiver_address = "0x739ff389c8eBd9339E69611d46Eec6212179BB67";
  const chain_id_optimism = "eip155:10";
  const chain_id_base = "eip155:8453";
  const usdc_contract_optimism = "0x0b2c639c533813f4aa9d7837caf62653d097ff85";

  let orchestration_id = "";

  it('bridging available', async () => {
    // Sending USDC to Optimism, but having the USDC balance on Base chain
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send,
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
    const amount_to_send_in_decimals = usdc_funds_on_base + 10_000_000
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
    const amount_to_send_in_decimals = usdc_funds_on_base
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
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send,
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
    // Expecting 3 transactions in the route
    expect(data.transactions.length).toBe(3)

    // First transaction expected to be the approval transaction
    const approvalTransaction = data.transactions[0]
    expect(approvalTransaction.chainId).toBe(chain_id_base)
    expect(approvalTransaction.nonce).not.toBe("0x00")
    expect(approvalTransaction.gas).toBe(gas_estimate)
    const decodedData = erc20Interface.decodeFunctionData('approve', approvalTransaction.data);  
    expect(decodedData.amount.toString()).toBe(amount_to_topup.toString().split('.')[0])


    // Second transaction expected to be the bridging to the Base
    expect(data.transactions[1].chainId).toBe(chain_id_base)
    expect(data.transactions[1].nonce).not.toBe("0x00")
    expect(data.transactions[1].gas).toBe(gas_estimate)

    // Last transaction expected to be the initial one
    expect(data.transactions[2].data).toBe(transactionObj.transaction.data)

    // Set the Orchestration ID for the next test
    orchestration_id = data.orchestrationId;
  })

  it('bridging status', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/ca/orchestrator/status?projectId=${projectId}&orchestrationId=${orchestration_id}`,
    )
    expect(resp.status).toBe(200)
    const data = resp.data
    expect(typeof data.status).toBe('string')
    expect(data.status).toBe('pending')
  })
})
