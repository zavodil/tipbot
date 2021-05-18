import 'regenerator-runtime/runtime'

const near = require('./rest-api-test-utils');
const utils = require('./utils');

const alice = "grant.testnet";
const bob = "place.testnet";
const admin = "zavodil.testnet";


//const contractName = await near.deploy("tipbot.wasm");

const deposit_size = 12.345;
const tip_size = 1;
const bob_contact = "kakoilogin";
const admin_commission = 0.003;

describe("Contract set", () => {
    test(process.env.REACT_CONTRACT_ID, async () => {
        expect(process.env.REACT_CONTRACT_ID).not.toBe(undefined)
    });
});

describe("Deposit and Withdraw", () => {
    test('Deposit', async () => {
        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});
        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBe(deposit_size);
    });

    test("Withdraw", async () => {
        const alice_wallet_balance_1 = await near.accountNearBalance(alice);
        const alice_deposit = await near.viewNearBalance("get_deposit", {account_id: alice});

        await near.call("withdraw", {}, {account_id: alice});

        const alice_deposit_3 = await near.viewNearBalance("get_deposit", {account_id: alice});
        expect(utils.RoundFloat(alice_deposit_3)).toBe(0);

        const alice_wallet_balance_2 = await near.accountNearBalance(alice);
        expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(alice_deposit, 1);
    });
});


describe("Tip, transfer tip to deposit", () => {
    test("Test Deposit", async () => {
        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});
        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBe(deposit_size);
    });

    test("Test Tip", async () => {
        const bob_balance_1 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact});

        await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });

        const bob_balance_2 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact});
        expect(utils.RoundFloat(bob_balance_2 - bob_balance_1)).toBe(tip_size);
    });

    test("Test transfer_tips_to_deposit", async () => {
        const bob_balance = await near.viewNearBalance("get_balance", {telegram_account: bob_contact});
        const bob_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: bob});

        await near.call("transfer_tips_to_deposit", {
            telegram_account: bob_contact,
            account_id: bob,
        }, {
            account_id: admin
        });

        const bob_balance_3 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact});
        expect(utils.RoundFloat(bob_balance_3)).toBe(0);

        const bob_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: bob});
        expect(utils.RoundFloat(bob_deposit_2 - bob_deposit_1)).toBeCloseTo(bob_balance - admin_commission, 5);
    });
});

describe("Deposit, tip and withdraw from telegram", () => {
    test("Withdraw from telegram", async () => {
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});

        await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });

        const bob_balance = await near.viewNearBalance("get_balance", {telegram_account: bob_contact});
        const bob_wallet_balance_1 = await near.accountNearBalance(bob);

        await near.call("withdraw_from_telegram", {
            telegram_account: bob_contact,
            account_id: bob,
        }, {
            account_id: admin
        });

        const bob_wallet_balance_2 = await near.accountNearBalance(bob);
        expect(utils.RoundFloat(bob_wallet_balance_2 - bob_wallet_balance_1)).toBe(bob_balance - admin_commission);
    });
});


describe("Deposit and Tip Too Much", () => {
    test("Tip throws an error after deposit and sending double tip", async () => {
        //deposit
        await near.call("withdraw", {}, {account_id: alice});
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});

        // send double tip
        const illegal = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact,
            amount: utils.ConvertToNear(deposit_size * 2),
        }, {
            account_id: alice
        });

        expect(illegal.type).toBe('FunctionCallError');
        expect(illegal.kind.ExecutionError).toMatch(/(Not enough tokens deposited to tip)/i);
    });
});

describe("Withdraw or Transfer by not an Admin", () => {
    test("Fail on withdraw_from_telegram from user", async () => {
        //deposit
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});

        await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });

        const illegal_withdraw = await near.call("withdraw_from_telegram", {
            telegram_account: bob_contact,
            account_id: alice,
        }, {
            account_id: alice
        });

        expect(illegal_withdraw.type).toBe('FunctionCallError');
        expect(illegal_withdraw.kind.ExecutionError).toMatch(/(No access)/i);
    });

    test("Fail on transfer_tips_to_deposit from user", async () => {
        const illegal_transfer =  await near.call("transfer_tips_to_deposit", {
            telegram_account: bob_contact,
            account_id: alice,
        }, {
            account_id: alice
        });

        expect(illegal_transfer.type).toBe('FunctionCallError');
        expect(illegal_transfer.kind.ExecutionError).toMatch(/(No access)/i);
    });
});