import { getTestSetup } from './init';
import { ethers, Interface } from "ethers"

describe('Chain abstraction orchestrator', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const erc20Interface = new Interface([
    'function transfer(address to, uint256 amount)',
    'function approve(address spender, uint256 amount) public returns (bool)'
  ]);

  // Default funding address
  const from_address_with_funds = "0x2aae531a81461f029cd55cb46703211c9227ba05";

  // Receiver address
  const receiver_address = "0x739ff389c8eBd9339E69611d46Eec6212179BB67";

  // Supported chains
  const chain_id_optimism = "eip155:10";
  const chain_id_base = "eip155:8453";
  const chain_id_arbitrum = "eip155:42161";
  
  // Current funds on different chains
  const usdc_token_symbol = "USDC";
  const usdc_funds = {};
  usdc_funds[chain_id_base] = 3_000_000;
  usdc_funds[chain_id_optimism] = 1_057_151;

  const usdt_token_symbol = "USDT";
  const usdt_funds = {};
  usdt_funds[chain_id_arbitrum] = 3_388_000;
  usdt_funds[chain_id_optimism] = 1_050_000;

  const usds_token_symbol = "USDS";
  const usds_funds = {};
  usds_funds[chain_id_optimism] = "902165684795715063"; // Using string amounts for USDS, as it has 18 decimals

  // Token decimals
  const token_decimals = {};
  token_decimals[usdc_token_symbol] = 6;
  token_decimals[usdt_token_symbol] = 6;
  token_decimals[usds_token_symbol] = 18;

  // Amount to send to Optimism
  const amount_to_send = 3_000_000
  
  // Asset contracts
  const usdc_contracts = {};
  usdc_contracts[chain_id_optimism] = "0x0b2c639c533813f4aa9d7837caf62653d097ff85";
  usdc_contracts[chain_id_base] = "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913";
  usdc_contracts[chain_id_arbitrum] = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831";
  const usdt_contracts = {};
  usdt_contracts[chain_id_optimism] = "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58";
  usdt_contracts[chain_id_arbitrum] = "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9";
  usdt_contracts[chain_id_base] = "0xfde4C96c8593536E31F229EA8f37b2ADa2699bb2";
  const usds_contracts = {};
  usds_contracts[chain_id_base] = "0x820c137fa70c8691f0e44dc420a5e53c168921dc";

  // Mutable variable to store the orchestrationId for getting status
  let orchestration_id = "";

  function checkRoutesResponse(
    response_object,
    sender_address,
    receiver_address,
    initial_tx_token_contract,
    initial_tx_token_symbol,
    initial_tx_amount,
    bridging_tx_token_chain_id,
    bridging_tx_token_contract,
    bridging_tx_token_symbol,
  ){
      expect(typeof response_object.orchestrationId).toBe('string')

      // Expecting 2 transactions in the route
      expect(response_object.transactions.length).toBe(2)

      // Check for the initialTransaction
      const initialTransaction = response_object.initialTransaction;
      expect(initialTransaction.from).toBe(sender_address.toLowerCase());
      expect(initialTransaction.to).toBe(initial_tx_token_contract.toLowerCase());
      expect(initialTransaction.gasLimit).not.toBe("0x00");

      // Check the initialTransaction metadata
      const initialTransactionMetadata = response_object.metadata.initialTransaction
      expect(initialTransactionMetadata.symbol).toBe(initial_tx_token_symbol)
      expect(initialTransactionMetadata.transferTo).toBe(receiver_address.toLowerCase())
      expect(initialTransactionMetadata.tokenContract).toBe(initial_tx_token_contract.toLowerCase())
      expect(BigInt(initialTransactionMetadata.amount)).toBe(BigInt(initial_tx_amount));
      expect(initialTransactionMetadata.decimals).toBe(token_decimals[initial_tx_token_symbol])

      // Check the metadata fundingFrom
      const fundingFrom = response_object.metadata.fundingFrom[0]
      expect(fundingFrom.chainId).toBe(bridging_tx_token_chain_id)
      expect(fundingFrom.symbol).toBe(bridging_tx_token_symbol)
      expect(fundingFrom.tokenContract).toBe(bridging_tx_token_contract.toLowerCase())
      expect(fundingFrom.decimals).toBe(token_decimals[bridging_tx_token_symbol])
      const bridging_amount = BigInt(fundingFrom.amount)

      // First transaction expected to be the approval transaction
      const approvalTransaction = response_object.transactions[0]
      expect(approvalTransaction.chainId).toBe(bridging_tx_token_chain_id)
      expect(approvalTransaction.nonce).not.toBe("0x00")
      expect(() => BigInt(approvalTransaction.gasLimit)).not.toThrow();
      const decodedData = erc20Interface.decodeFunctionData('approve', approvalTransaction.input);
      if (BigInt(decodedData.amount) < bridging_amount) {
        throw new Error(`Expected approval amount is incorrect`);
      }

      // Second transaction expected to be the bridging to the Base
      const bridgingTransaction = response_object.transactions[1]
      expect(bridgingTransaction.chainId).toBe(bridging_tx_token_chain_id)
      expect(bridgingTransaction.nonce).not.toBe("0x00")
      expect(() => BigInt(approvalTransaction.gasLimit)).not.toThrow();

      // Check the metadata checkIn
      expect(typeof response_object.metadata.checkIn).toBe('number')

      return response_object.orchestrationId;
  }

  it('bridging unavailable (insufficient funds)', async () => {
    // Having the USDC balance on Base chain less then the amount to send
    const amount_to_send_in_decimals = usdc_funds[chain_id_base] + 10_000_000
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send_in_decimals,
    ]);

    const transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contracts[chain_id_optimism],
        value: "0x00", // Zero native tokens
        input: data_encoded,
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
    const amount_to_send_in_decimals = usdc_funds[chain_id_base]
    const empty_wallet_address = ethers.Wallet.createRandom().address
    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send_in_decimals,
    ]);

    const transactionObj = {
      transaction: {
        from: empty_wallet_address,
        to: usdc_contracts[chain_id_optimism],
        value: "0x00", // Zero native tokens
        input: data_encoded,
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

    const transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contracts[chain_id_optimism],
        value: "0x00", // Zero native tokens
        input: data_encoded,
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

  it('bridging routes (USDC Base → USDC Optimism)', async () => {
    // Sending USDC to Optimism, but having the balance of USDC on Base chain
    // which expected to be used for bridging

    // How much needs to be topped up
    const amount_to_topup = Math.round(amount_to_send - usdc_funds[chain_id_optimism]);

    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send,
    ]);

    const transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contracts[chain_id_optimism],
        value: "0x00", // Zero native tokens
        input: data_encoded,
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)

    orchestration_id = checkRoutesResponse(
      resp.data,
      from_address_with_funds,
      receiver_address,
      usdc_contracts[chain_id_optimism],
      usdc_token_symbol,
      amount_to_send,
      chain_id_base,
      usdc_contracts[chain_id_base],
      usdc_token_symbol,
    )
  })

  it('bridging routes (USDT Arbitrum → USDT Optimism)', async () => {
    // Sending USDT to Optimism, but having the USDT balance on Arbitrum.

    // How much needs to be topped up
    const amount_to_topup = Math.round(amount_to_send - usdt_funds[chain_id_optimism]);

    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send,
    ]);

    const transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdt_contracts[chain_id_optimism],
        value: "0x00", // Zero native tokens
        input: data_encoded,
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)

    orchestration_id = checkRoutesResponse(
      resp.data,
      from_address_with_funds,
      receiver_address,
      usdt_contracts[chain_id_optimism],
      usdt_token_symbol,
      amount_to_send,
      chain_id_arbitrum,
      usdt_contracts[chain_id_arbitrum],
      usdt_token_symbol,
    )
  })

  it('bridging routes (USDT Optimism → USDT Arbitrum)', async () => {
    // Sending USDT on Arbitrum, but having the USDT balance on Optimism.
    const amount_to_send = usdt_funds[chain_id_arbitrum] + 265_000;

    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send,
    ]);

    const transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdt_contracts[chain_id_arbitrum],
        value: "0x00", // Zero native tokens
        input: data_encoded,
        chainId: chain_id_arbitrum,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)

    orchestration_id = checkRoutesResponse(
      resp.data,
      from_address_with_funds,
      receiver_address,
      usdt_contracts[chain_id_arbitrum],
      usdt_token_symbol,
      amount_to_send,
      chain_id_optimism,
      usdt_contracts[chain_id_optimism],
      usdt_token_symbol,
    )
  })

  it('bridging routes (USDC Base → USDS Base)', async () => {
    // Override the default address to source from the USDC Base only.
    const from_address_with_funds = "0xe6f8b93B0eed834816C5aDd2aA0989e2fF97616c";
    // Sending USDS on Base, but having the USDC balance on Base.
    const amount_to_send = "2000005684795715100";

    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send,
    ]);

    const transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usds_contracts[chain_id_base],
        value: "0x00", // Zero native tokens
        input: data_encoded,
        chainId: chain_id_base,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)

    orchestration_id = checkRoutesResponse(
      resp.data,
      from_address_with_funds,
      receiver_address,
      usds_contracts[chain_id_base],
      usds_token_symbol,
      amount_to_send,
      chain_id_base,
      usdc_contracts[chain_id_base],
      usdc_token_symbol,
    )
  })

  it('bridging routes (USDS Base → USDC Optimism)', async () => {
    // Override the default address to source from the USDS Base only.
    const from_address_with_funds = "0xFB85fBfF17B35C3c2889Bcec1D38cf3B8Bb228e0";
    // Sending USDC on Optimims, but having the USDS balance on Base.
    const amount_to_send = "600000";

    const data_encoded = erc20Interface.encodeFunctionData('transfer', [
      receiver_address,
      amount_to_send,
    ]);

    const transactionObj = {
      transaction: {
        from: from_address_with_funds,
        to: usdc_contracts[chain_id_optimism],
        value: "0x00", // Zero native tokens
        input: data_encoded,
        chainId: chain_id_optimism,
      }
    }

    let resp: any = await httpClient.post(
      `${baseUrl}/v1/ca/orchestrator/route?projectId=${projectId}`,
      transactionObj
    )
    expect(resp.status).toBe(200)

    orchestration_id = checkRoutesResponse(
      resp.data,
      from_address_with_funds,
      receiver_address,
      usdc_contracts[chain_id_optimism],
      usdc_token_symbol,
      amount_to_send,
      chain_id_base,
      usds_contracts[chain_id_base],
      usds_token_symbol,
    )

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
