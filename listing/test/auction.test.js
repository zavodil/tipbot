import 'regenerator-runtime/runtime'

const contract = require('./rest-api-test-utils');
const utils = require('./utils');
const helper = require('./helper');

const alice = "test_alice.testnet";
const bob = "test_bob.testnet";
const carol = "test_carol.testnet";
const admin = "zavodil2.testnet";

const tiptoken_account_id = "tiptoken.testnet";
const ft_contract_account_id_1 = "usdt.fakes.testnet";
const ft_contract_account_id_2 = "usdc.fakes.testnet";
const ft_contract_account_id_3 = "dai.fakes.testnet";
const listing_payment = 0.1;
const storage_deposit_payment_payment = 0.02;

const ft_deposit_size = 1000000;

const contract_id = process.env.LISTING_CONTRACT_ID;
const near = new contract(contract_id);

const ft_1 = new contract(ft_contract_account_id_1);
const ft_2 = new contract(ft_contract_account_id_2);
const ft_3= new contract(ft_contract_account_id_3);
const tt = new contract(tiptoken_account_id);


describe("ContractSet", () => {
    test("Contract is not null " + helper.GetContractUrl(), async () => {
        expect(contract_id).not.toBe(undefined)
    });

    test("Init contract", async () => {
        await near.call("new", {
            owner_id: admin,
            tiptoken_account_id,
            listing_payment,
        }, {account_id: contract_id, log_errors: false});

        // ft storage
        await ft_1.call("storage_deposit", {}, {account_id: alice, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_1.call("storage_deposit", {}, {account_id: bob, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_1.call("storage_deposit", {}, {account_id: carol, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_2.call("storage_deposit", {}, {account_id: alice, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_2.call("storage_deposit", {}, {account_id: bob, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_2.call("storage_deposit", {}, {account_id: carol, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_3.call("storage_deposit", {}, {account_id: alice, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_3.call("storage_deposit", {}, {account_id: bob, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});
        await ft_3.call("storage_deposit", {}, {account_id: carol, attached_tokens: utils.ConvertToNear(storage_deposit_payment_payment)});

        // deposit ft
        await tt.call("ft_transfer", {receiver_id: alice, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await tt.call("ft_transfer", {receiver_id: bob, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await tt.call("ft_transfer", {receiver_id: carol, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_1.call("ft_transfer", {receiver_id: alice, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_1.call("ft_transfer", {receiver_id: bob, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_1.call("ft_transfer", {receiver_id: carol, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_2.call("ft_transfer", {receiver_id: alice, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_2.call("ft_transfer", {receiver_id: bob, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_2.call("ft_transfer", {receiver_id: carol, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_3.call("ft_transfer", {receiver_id: alice, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_3.call("ft_transfer", {receiver_id: bob, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});
        await ft_3.call("ft_transfer", {receiver_id: carol, amount: ft_deposit_size.toString(), msg: ""},
            {account_id: admin, attached_tokens: 1});

    });

    test('Accounts has enough funds', async () => {
        const alice_wallet_balance = await near.accountNearBalance(alice);
        expect(alice_wallet_balance).toBeGreaterThan(20);

        const bob_wallet_balance = await near.accountNearBalance(bob);
        expect(bob_wallet_balance).toBeGreaterThan(20);
    });
});
describe("CreateAuction", () => {
    test("Reset existing tokens", async () => {
        let remove_existing_token_1 = await near.call("remove_existing_token", {
            token_id: ft_contract_account_id_1,
        }, {account_id: admin, log_errors: true});
        expect(remove_existing_token_1.type).not.toBe('FunctionCallError');

        let remove_existing_token_2 = await near.call("remove_existing_token", {
            token_id: ft_contract_account_id_2,
        }, {account_id: admin, log_errors: true});
        expect(remove_existing_token_2.type).not.toBe('FunctionCallError');

        let remove_existing_token_3 = await near.call("remove_existing_token", {
            token_id: ft_contract_account_id_3,
        }, {account_id: admin, log_errors: true});
        expect(remove_existing_token_3.type).not.toBe('FunctionCallError');
    });

    test("add_listing_auction", async () => {
        const next_auction_id_1 = await near.view("get_next_auction_id", {});

        let add_listing_auction = await near.call("add_listing_auction", {
            end_date: utils.GetTimestamp(30),
            unlock_date_for_winner: utils.GetTimestamp(60),
        }, {account_id: admin, log_errors: true});
        expect(add_listing_auction.type).not.toBe('FunctionCallError');

        const next_auction_id_2 = await near.view("get_next_auction_id", {});
        expect(next_auction_id_2 - next_auction_id_1).toBe(1);
    });

    test ("add_token_to_auction", async () => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        const get_auction_tokens_number_1 = await near.view("get_auction_tokens_number", {auction_id}, {return_value_int: true});
        expect(get_auction_tokens_number_1).toBe(0);

        let add_token_to_auction_1 = await near.call("add_token_to_auction", {
            auction_id,
            token_id: ft_contract_account_id_1,
        }, {account_id: alice, log_errors: false, deposit_near: listing_payment + storage_deposit_payment_payment, attached_gas: 200000000000000});
        expect(add_token_to_auction_1.type).not.toBe('FunctionCallError');

        let add_token_to_auction_2 = await near.call("add_token_to_auction", {
            auction_id,
            token_id: ft_contract_account_id_2,
        }, {account_id: bob, log_errors: false, deposit_near: listing_payment + storage_deposit_payment_payment, attached_gas: 200000000000000});
        expect(add_token_to_auction_2.type).not.toBe('FunctionCallError');

        let add_token_to_auction_3 = await near.call("add_token_to_auction", {
            auction_id,
            token_id: ft_contract_account_id_3,
        }, {account_id: carol, log_errors: false, deposit_near: listing_payment + storage_deposit_payment_payment, attached_gas: 200000000000000});
        expect(add_token_to_auction_3.type).not.toBe('FunctionCallError');

        const get_auction_tokens_number_2 = await near.view("get_auction_tokens_number", {auction_id}, {return_value_int: true});
        expect(get_auction_tokens_number_2).toBe(3);
    });

    let deposit_ft_1_1 = 100; // alice
    let deposit_ft_1_2 = 200; // bob
    let deposit_ft_2_1 = 100; // alice
    let deposit_ft_2_2 = 250; // bob
    let deposit_ft_3_1 = 100; // alice
    let deposit_ft_3_2 = 100; // alice
    let deposit_ft_3_3 = 100; // carol

    test ("Vote for tokens on auction", async () => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        await tt.call("ft_transfer_call", {
                receiver_id: contract_id,
                amount: deposit_ft_1_1.toString(),
                msg: `{"Vote": {"auction_id": ${auction_id}, "listing_token_id": "${ft_contract_account_id_1}"} }`
            },
            {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});
        await tt.call("ft_transfer_call", {
                receiver_id: contract_id,
                amount: deposit_ft_1_2.toString(),
                msg: `{"Vote": {"auction_id": ${auction_id}, "listing_token_id": "${ft_contract_account_id_1}"} }`
            },
            {account_id: bob, attached_tokens: 1, attached_gas: 35000000000000});

        await tt.call("ft_transfer_call", {
                receiver_id: contract_id,
                amount: deposit_ft_2_1.toString(),
                msg: `{"Vote": {"auction_id": ${auction_id}, "listing_token_id": "${ft_contract_account_id_2}"} }`
            },
            {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});
        await tt.call("ft_transfer_call", {
                receiver_id: contract_id,
                amount: deposit_ft_2_2.toString(),
                msg: `{"Vote": {"auction_id": ${auction_id}, "listing_token_id": "${ft_contract_account_id_2}"} }`
            },
            {account_id: bob, attached_tokens: 1, attached_gas: 35000000000000});

        await tt.call("ft_transfer_call", {
                receiver_id: contract_id,
                amount: deposit_ft_3_1.toString(),
                msg: `{"Vote": {"auction_id": ${auction_id}, "listing_token_id": "${ft_contract_account_id_3}"} }`
            },
            {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});
        await tt.call("ft_transfer_call", {
                receiver_id: contract_id,
                amount: deposit_ft_3_2.toString(),
                msg: `{"Vote": {"auction_id": ${auction_id}, "listing_token_id": "${ft_contract_account_id_3}"} }`
            },
            {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});
        await tt.call("ft_transfer_call", {
                receiver_id: contract_id,
                amount: deposit_ft_3_3.toString(),
                msg: `{"Vote": {"auction_id": ${auction_id}, "listing_token_id": "${ft_contract_account_id_3}"} }`
            },
            {account_id: carol, attached_tokens: 1, attached_gas: 35000000000000});

        // TOTAL DEPOSITS
        const auction_total_deposit_ft_1 = await near.view("get_auction_total_deposit", {
            auction_id,
            token_id: ft_contract_account_id_1
        }, {return_value_int: true});
        expect(auction_total_deposit_ft_1).toBe(deposit_ft_1_1 + deposit_ft_1_2);
        const auction_total_deposit_ft_2 = await near.view("get_auction_total_deposit", {
            auction_id,
            token_id: ft_contract_account_id_2
        }, {return_value_int: true});
        expect(auction_total_deposit_ft_2).toBe(deposit_ft_2_1 + deposit_ft_2_2);
        const auction_total_deposit_ft_3 = await near.view("get_auction_total_deposit", {
            auction_id,
            token_id: ft_contract_account_id_3
        }, {return_value_int: true});
        expect(auction_total_deposit_ft_3).toBe(deposit_ft_3_1 + deposit_ft_3_2 + deposit_ft_3_3);

        // USER DEPOSITS
        const alice_deposit_ft_1 = await near.view("get_user_deposit",
            {auction_id, account_id: alice, token_id: ft_contract_account_id_1}, {return_value_int: true});
        expect(alice_deposit_ft_1).toBe(deposit_ft_1_1);

        const bob_deposit_ft_1 = await near.view("get_user_deposit",
            {auction_id, account_id: bob, token_id: ft_contract_account_id_1}, {return_value_int: true});
        expect(bob_deposit_ft_1).toBe(deposit_ft_1_2);

        const carol_deposit_ft_1 = await near.view("get_user_deposit",
            {auction_id, account_id: carol, token_id: ft_contract_account_id_1}, {return_value_int: true});
        expect(carol_deposit_ft_1).toBe(0);

        const alice_deposit_ft_3 = await near.view("get_user_deposit",
            {auction_id, account_id: alice, token_id: ft_contract_account_id_3}, {return_value_int: true});
        expect(alice_deposit_ft_3).toBe(deposit_ft_3_1 + deposit_ft_3_2);

    });

    const reward_ft_1_1 = 1000; // alice
    const reward_ft_1_2 = 500; // bob
    const reward_ft_2_1 = 200; // alice
    const reward_ft_2_2 = 200; // carol
    const reward_ft_3_1 = 300; // alice
    const reward_ft_3_2 = 100; // alice
    const reward_ft_3_3 = 50; // carol

    test("Add rewards", async () => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        await ft_1.call("ft_transfer_call", {receiver_id: contract_id,
                amount: reward_ft_1_1.toString(),
                msg: `{"AddReward": {"auction_id": ${auction_id}} }`
            }, {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});

        await ft_1.call("ft_transfer_call", {receiver_id: contract_id,
            amount: reward_ft_1_2.toString(),
            msg: `{"AddReward": {"auction_id": ${auction_id}} }`
        }, {account_id: bob, attached_tokens: 1, attached_gas: 35000000000000});

        await ft_2.call("ft_transfer_call", {receiver_id: contract_id,
            amount: reward_ft_2_1.toString(),
            msg: `{"AddReward": {"auction_id": ${auction_id}} }`
        }, {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});

        await ft_2.call("ft_transfer_call", {receiver_id: contract_id,
            amount: reward_ft_2_2.toString(),
            msg: `{"AddReward": {"auction_id": ${auction_id}} }`
        }, {account_id: carol, attached_tokens: 1, attached_gas: 35000000000000});

        await ft_3.call("ft_transfer_call", {receiver_id: contract_id,
            amount: reward_ft_3_1.toString(),
            msg: `{"AddReward": {"auction_id": ${auction_id}} }`
        }, {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});

        await ft_3.call("ft_transfer_call", {receiver_id: contract_id,
            amount: reward_ft_3_2.toString(),
            msg: `{"AddReward": {"auction_id": ${auction_id}} }`
        }, {account_id: alice, attached_tokens: 1, attached_gas: 35000000000000});

        await ft_3.call("ft_transfer_call", {receiver_id: contract_id,
            amount: reward_ft_3_3.toString(),
            msg: `{"AddReward": {"auction_id": ${auction_id}} }`
        }, {account_id: carol, attached_tokens: 1, attached_gas: 35000000000000});

        const auction_total_rewards_ft_1 = await near.view("get_auction_total_reward", {
            auction_id,
            token_id: ft_contract_account_id_1
        }, {return_value_int: true});
        expect(auction_total_rewards_ft_1).toBe(reward_ft_1_1 + reward_ft_1_2);

        const auction_total_rewards_ft_2 = await near.view("get_auction_total_reward", {
            auction_id,
            token_id: ft_contract_account_id_2
        }, {return_value_int: true});
        expect(auction_total_rewards_ft_2).toBe(reward_ft_2_1 + reward_ft_2_2);

        const auction_total_rewards_ft_3 = await near.view("get_auction_total_reward", {
            auction_id,
            token_id: ft_contract_account_id_3
        }, {return_value_int: true});
        expect(auction_total_rewards_ft_3).toBe(reward_ft_3_1 + reward_ft_3_2 + reward_ft_3_3);
    });

    test ("Finalize auction", async () => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        let cheat_update_auction_end_date = await near.call("set_auction_end_date", {
            auction_id,
            end_date: utils.GetTimestamp(0),
        }, {account_id: admin, log_errors: true});
        expect(cheat_update_auction_end_date.type).not.toBe('FunctionCallError');

        let finalize = await near.call("finalize", {
            auction_id,
        }, {account_id: alice, log_errors: true});
        expect(finalize.type).not.toBe('FunctionCallError');


        const get_auction_winner = await near.view("get_auction_winner", {auction_id}, {return_value: true});
        expect(get_auction_winner).toBe(ft_contract_account_id_2);
    });

    /*
   const reward_ft_1_1 = 1000; // alice
const reward_ft_1_2 = 500; // bob
const reward_ft_2_1 = 200; // alice
const reward_ft_2_2 = 200; // carol
const reward_ft_3_1 = 300; // alice
const reward_ft_3_2 = 100; // alice
const reward_ft_3_3 = 50; // carol*/

    test("Claim rewards", async() => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        const alice_ft_2_balance_1 = await ft_2.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        const alice_claim_reward = await near.call("claim_reward", {
            auction_id,
        }, {account_id: alice, log_errors: true});
        expect(alice_claim_reward.type).not.toBe('FunctionCallError');
        const alice_ft_2_balance_2 = await ft_2.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_ft_2_balance_2 - alice_ft_2_balance_1).toBeCloseTo(utils.RoundFloat((reward_ft_2_1 + reward_ft_2_2) * deposit_ft_2_1 / (deposit_ft_2_1 + deposit_ft_2_2)), -1);

        const alice_claim_reward_illegal = await near.call("claim_reward", {
            auction_id,
        }, {account_id: alice});
        //expect(alice_claim_reward_illegal.type).toBe('FunctionCallError');
        expect(alice_claim_reward_illegal.kind.ExecutionError).toMatch(/ERR_REWARD_ALREADY_CLAIMED/i);

        const bob_ft_2_balance_1 = await ft_2.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        const bob_claim_reward = await near.call("claim_reward", {
            auction_id,
        }, {account_id: bob, log_errors: true});
        expect(bob_claim_reward.type).not.toBe('FunctionCallError');
        const bob_ft_2_balance_2 = await ft_2.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        expect(bob_ft_2_balance_2 - bob_ft_2_balance_1).toBeCloseTo(utils.RoundFloat((reward_ft_2_1 + reward_ft_2_2) * deposit_ft_2_2 / (deposit_ft_2_1 + deposit_ft_2_2)), -1);

        const bob_claim_reward_illegal = await near.call("claim_reward", {
            auction_id,
        }, {account_id: bob});
        expect(bob_claim_reward_illegal.type).toBe('FunctionCallError');
        expect(bob_claim_reward_illegal.kind.ExecutionError).toMatch(/ERR_REWARD_ALREADY_CLAIMED/i);

        const carol_claim_reward_illegal_no_reward = await near.call("claim_reward", {
            auction_id,
        }, {account_id: carol});
        expect(carol_claim_reward_illegal_no_reward.type).toBe('FunctionCallError');
        expect(carol_claim_reward_illegal_no_reward.kind.ExecutionError).toMatch(/ERR_NOTHING_TO_CLAIM/i);
    });


    test("Looser withdraw bids", async () => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        // ALICE FT_1
        const alice_tt_balance_1 = await tt.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        const withdraw_bids_alice_ft_1 = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_1,
        }, {account_id: alice, log_errors: true});
        expect(withdraw_bids_alice_ft_1.type).not.toBe('FunctionCallError');
        const alice_tt_balance_2 = await tt.view("ft_balance_of", {account_id: alice, delay: 100}, {return_value_int: true});
        expect(alice_tt_balance_2 - alice_tt_balance_1).toBe(deposit_ft_1_1);

        // ALICE FT_3 - 2 BIDS
        const withdraw_bids_alice_ft_3 = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_3,
        }, {account_id: alice, log_errors: true});
        expect(withdraw_bids_alice_ft_3.type).not.toBe('FunctionCallError');
        const alice_tt_balance_3 = await tt.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_tt_balance_3 - alice_tt_balance_2).toBe(deposit_ft_3_1 + deposit_ft_3_2);

        // ALICE FT_3 ILLEGAL (AGAIN)
        const withdraw_bids_alice_illegal = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_3,
        }, {account_id: alice});
        expect(withdraw_bids_alice_illegal.type).toBe('FunctionCallError');
        expect(withdraw_bids_alice_illegal.kind.ExecutionError).toMatch(/ERR_NOTHING_TO_CLAIM/i);

        // BOB FT_1
        const bob_tt_balance_1 = await tt.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        const withdraw_bids_bob_ft_1 = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_1,
        }, {account_id: bob, log_errors: true});
        expect(withdraw_bids_bob_ft_1.type).not.toBe('FunctionCallError');
        const bob_tt_balance_2 = await tt.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        expect(bob_tt_balance_2 - bob_tt_balance_1).toBe(deposit_ft_1_2);
        // BOB FT_3 - NONE
        const withdraw_bids_bob_ft_2 = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_3,
        }, {account_id: bob});
        expect(withdraw_bids_bob_ft_2.type).toBe('FunctionCallError');
        expect(withdraw_bids_bob_ft_2.kind.ExecutionError).toMatch(/ERR_NOTHING_TO_CLAIM/i);
        const bob_tt_balance_3 = await tt.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        expect(bob_tt_balance_3 - bob_tt_balance_2).toBe(0);
    });

    test("Looser withdraw rewards", async () => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        // ALICE FT_1
        const alice_ft_1_balance_1 = await ft_1.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        const withdraw_rewards_alice_ft_1 = await near.call("withdraw_rewards", {
            auction_id,
            token_id: ft_contract_account_id_1,
        }, {account_id: alice, log_errors: true});
        expect(withdraw_rewards_alice_ft_1.type).not.toBe('FunctionCallError');
        const alice_ft_1_balance_2 = await ft_1.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_ft_1_balance_2 - alice_ft_1_balance_1).toBe(reward_ft_1_1);

        // ALICE FT_1 ILLEGAL (AGAIN)
        const withdraw_rewards_alice_illegal = await near.call("withdraw_rewards", {
            auction_id,
            token_id: ft_contract_account_id_1,
        }, {account_id: alice});
        expect(withdraw_rewards_alice_illegal.type).toBe('FunctionCallError');
        expect(withdraw_rewards_alice_illegal.kind.ExecutionError).toMatch(/ERR_NOTHING_TO_WITHDRAW/i);
        const alice_ft_1_balance_2_illegal = await ft_1.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_ft_1_balance_2_illegal - alice_ft_1_balance_2).toBe(0);

        // ALICE FT_3 2 REWARDS
        const alice_ft_3_balance_1 = await ft_3.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        const withdraw_rewards_alice_ft_3 = await near.call("withdraw_rewards", {
            auction_id,
            token_id: ft_contract_account_id_3,
        }, {account_id: alice, log_errors: true});
        expect(withdraw_rewards_alice_ft_3.type).not.toBe('FunctionCallError');
        const alice_ft_3_balance_2 = await ft_3.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_ft_3_balance_2 - alice_ft_3_balance_1).toBe(reward_ft_3_1 + reward_ft_3_2);

        // BOB FT_1
        const bob_ft_1_balance_1 = await ft_1.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        const withdraw_rewards_bob_ft_1 = await near.call("withdraw_rewards", {
            auction_id,
            token_id: ft_contract_account_id_1,
        }, {account_id: bob, log_errors: true});
        expect(withdraw_rewards_bob_ft_1.type).not.toBe('FunctionCallError');
        const bob_ft_1_balance_2 = await ft_1.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        expect(bob_ft_1_balance_2 - bob_ft_1_balance_1).toBe(reward_ft_1_2);

        // CAROL FT_1 - NO REWARD
        const carol_ft_1_balance_1 = await ft_1.view("ft_balance_of", {account_id: carol}, {return_value_int: true});
        const withdraw_rewards_carol_ft_1 = await near.call("withdraw_rewards", {
            auction_id,
            token_id: ft_contract_account_id_1,
        }, {account_id: carol});
        expect(withdraw_rewards_carol_ft_1.type).toBe('FunctionCallError');
        const carol_ft_1_balance_2 = await ft_1.view("ft_balance_of", {account_id: carol}, {return_value_int: true});
        expect(carol_ft_1_balance_2 - carol_ft_1_balance_1).toBe(0);

        // ALICE FT_2 - NO REWARD, WINNER
        const alice_ft_2_balance_1 = await ft_2.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        const withdraw_rewards_alice_ft_2 = await near.call("withdraw_rewards", {
            auction_id,
            token_id: ft_contract_account_id_2,
        }, {account_id: alice});
        expect(withdraw_rewards_alice_ft_2.type).toBe('FunctionCallError');
        const alice_ft_2_balance_2 = await ft_2.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_ft_2_balance_2 - alice_ft_2_balance_1).toBe(0);
    });


    test("Withdraw winner bid", async () => {
        const auction_id = await near.view("get_next_auction_id", {}) - 1;
        expect(auction_id).toBeGreaterThan(0);

        const alice_tt_balance_1 = await tt.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        const withdraw_winner_bids_alice_too_early = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_2,
        }, {account_id: alice});
        expect(withdraw_winner_bids_alice_too_early.type).toBe('FunctionCallError');
        const alice_tt_balance_2 = await tt.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_tt_balance_2 - alice_tt_balance_1).toBe(0);

        let set_auction_unlock_date_for_winner = await near.call("set_auction_unlock_date_for_winner", {
            auction_id,
            unlock_date_for_winner: utils.GetTimestamp(0),
        }, {account_id: admin, log_errors: true});
        expect(set_auction_unlock_date_for_winner.type).not.toBe('FunctionCallError');

        const withdraw_winner_bids_alice = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_2,
        }, {account_id: alice, log_errors: true});
        expect(withdraw_winner_bids_alice.type).not.toBe('FunctionCallError');
        const alice_tt_balance_3 = await tt.view("ft_balance_of", {account_id: alice}, {return_value_int: true});
        expect(alice_tt_balance_3 - alice_tt_balance_2).toBe(deposit_ft_2_1);

        const bob_tt_balance_1 = await tt.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        const withdraw_winner_bids_bob = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_2,
        }, {account_id: bob, log_errors: true});
        expect(withdraw_winner_bids_bob.type).not.toBe('FunctionCallError');
        const bob_tt_balance_2 = await tt.view("ft_balance_of", {account_id: bob}, {return_value_int: true});
        expect(bob_tt_balance_2 - bob_tt_balance_1).toBe(deposit_ft_2_2);

        const carol_tt_balance_1 = await tt.view("ft_balance_of", {account_id: carol}, {return_value_int: true});
        const withdraw_winner_bids_carol_no_reward = await near.call("withdraw_bids", {
            auction_id,
            token_id: ft_contract_account_id_2,
        }, {account_id: carol});
        expect(withdraw_winner_bids_carol_no_reward.type).toBe('FunctionCallError');
        expect(withdraw_winner_bids_carol_no_reward.kind.ExecutionError).toMatch(/ERR_NOTHING_TO_CLAIM/i);
        const carol_tt_balance_2 = await tt.view("ft_balance_of", {account_id: carol}, {return_value_int: true});
        expect(carol_tt_balance_2 - carol_tt_balance_1).toBe(0);

    });



});
