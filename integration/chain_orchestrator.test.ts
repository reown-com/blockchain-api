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
  const usdc_token_symbol = "USDC";
  // Amount to send to Optimism
  const amount_to_send = 3_000_000
  // Amount bridging multiplier
  const amount_multiplier = 5; // +5% topup
  // Minimal bridging fees covering
  const minimal_bridging_fees_covering = 50_000; // 0.05 USDC/USDT
  // How much needs to be topped up
  const amount_to_topup = Math.round((amount_to_send - usdc_funds_on_optimism) * (100 + amount_multiplier) / 100);
  // Default gas esimation is default with 6x increase
  const gas_estimate = "0xf9e82";

  const receiver_address = "0x739ff389c8eBd9339E69611d46Eec6212179BB67";
  const chain_id_optimism = "eip155:10";
  const usdc_contract_optimism = "0x0b2c639c533813f4aa9d7837caf62653d097ff85";
  const chain_id_base = "eip155:8453";
  const usdc_contract_base = "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913";

  let orchestration_id = "";

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
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)
    expect(resp.data.error).toBe("INSUFFICIENT_FUNDS")
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
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)
    expect(resp.data.error).toBe("INSUFFICIENT_FUNDS")
  })

  it('bridging routes (no bridging needed)', async () => {
    // Sending USDC to Optimism, having the USDC balance on Base chain
    const amount_to_send_in_decimals = 20_000 // Less then bridging needed amount
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
    expect(resp.data.transactions.length).toBe(0)

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
    // Expecting 2 transactions in the route
    expect(data.transactions.length).toBe(2)

    // First transaction expected to be the approval transaction
    const approvalTransaction = data.transactions[0]
    expect(approvalTransaction.chainId).toBe(chain_id_base)
    expect(approvalTransaction.nonce).not.toBe("0x00")
    expect(approvalTransaction.gas).toBe(gas_estimate)
    const decodedData = erc20Interface.decodeFunctionData('approve', approvalTransaction.data);  
    expect(decodedData.amount.toString()).toBe(amount_to_topup.toString())

    // Second transaction expected to be the bridging to the Base
    const bridgingTransaction = data.transactions[1]
    expect(bridgingTransaction.chainId).toBe(chain_id_base)
    expect(bridgingTransaction.nonce).not.toBe("0x00")
    expect(bridgingTransaction.gas).toBe(gas_estimate)

    // Check the metadata fundingFrom
    const fundingFrom = data.metadata.fundingFrom[0]
    expect(fundingFrom.chainId).toBe(chain_id_base)
    expect(fundingFrom.symbol).toBe(usdc_token_symbol)
    expect(fundingFrom.tokenContract).toBe(usdc_contract_base)
    expect(fundingFrom.amount).toBe("0x" + amount_to_topup.toString(16))

    // Check the metadata checkIn
    expect(typeof data.metadata.checkIn).toBe('number')

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
    expect(data.status).toBe('PENDING')
    expect(data.checkIn).toBe(3000)
  })
})
