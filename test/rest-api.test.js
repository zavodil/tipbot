import 'regenerator-runtime/runtime'

const contract = require('./rest-api-test-utils');
const utils = require('./utils');
const nearAPI = require("near-api-js");

const tipbot_account_id = process.env.CONTRACT_NAME;
console.log(tipbot_account_id);


const alice = "grant.testnet";
const alice_contact_handler = "alice_contact_01";
const alice_contact_id = 1234;
const alice_chat_id = Date.now();
const bob = "place.testnet";
const bob_contact_handler = "bob_contact_02";
const bob_contact_id = 4565;
const admin = "nearsocial.testnet";
const carol = "zavodil2.testnet"; // without contacts
const carol_contact = "";
const carol_contact_id = 9876543210;

const chat_id = 9999;
const chat_admin = bob;
const treasure_fee_numerator = 10;
const chat_tiptoken_fraction = 0.4;
const sender_tiptoken_fraction = 0.4;


const ft_contract_account_id = "dai.fakes.testnet";
const tiptoken_account_id = "tiptoken.testnet";
const wrap_near_contract_id = "wrap.testnet";

const tiptoken_owner_account_id = "zavodil2.testnet";

const deposit_size = 4.678;
const tip_size = 0.5234;
const withdraw_near_commission = 0.003;
const withdraw_ft_commission = 0.003;

const treasury_fee_numerator = 5;
const treasury_fee_denominator = 100;
const treasury_fee = treasury_fee_numerator / treasury_fee_denominator;

const service_fee_numerator = 1;
const service_fee_denominator = 100;
const service_fee = service_fee_numerator / service_fee_denominator;

const ft_deposit_size = ConvertToPow18(3.567);
const ft_tip_size = ConvertToPow18(0.345);
const ft_admin_commission = ConvertToPow18(0);

//const ft_deposit_size = ConvertToPow6(4.567);
//const ft_tip_size = ConvertToPow6(0.345);


function ConvertToPow18(amount) {
    return (Math.round(amount * 100000000)).toString() + "0000000000";
}

function ConvertToPow6(amount) {
    return (Math.round(amount * 1000000)).toString();
}

const near = new contract(tipbot_account_id);

function getConfig() {
    return {
        owner_id: admin,
        operator_id: admin,

        withdraw_available: true,
        tip_available: true,

        // part of every tip goes to treasury
        treasury_fee: {
            numerator: treasury_fee_numerator,
            denominator: treasury_fee_denominator
        },
        // part of treasury_fee goes to service
        service_fee: {
            numerator: service_fee_numerator,
            denominator: service_fee_denominator
        },

        tiptoken_account_id: tiptoken_account_id,
        wrap_near_contract_id
    }
}

describe("Set", () => {
    test("Contract is not null " + tipbot_account_id, async () => {
        //const contractName = await near.deploy("tipbot.wasm");
        expect(tipbot_account_id).not.toBe(undefined)
    });

    test("Init contract", async () => {
        await near.call("new", {
            config: getConfig()
        }, {account_id: tipbot_account_id});

        const ft = new contract(ft_contract_account_id);
        await ft.call("storage_deposit", {}, {account_id: tipbot_account_id, tokens: utils.ConvertToNear(0.2)});
        await ft.call("storage_deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(0.2)});
        await ft.call("storage_deposit", {}, {account_id: bob, tokens: utils.ConvertToNear(0.2)});

        let whitelist_token_by_user = await near.call("whitelist_token", {
                tips_available: true,
                token_id: null,
                min_deposit: "100000000000000000000000",
                min_tip: "10000000000000000000000",
                withdraw_commission: nearAPI.utils.format.parseNearAmount(withdraw_near_commission.toString()),
                dex: null,
                swap_contract_id: null,
                swap_pool_id: null
            }
            , {account_id: alice});
        expect(whitelist_token_by_user.type).toBe('FunctionCallError');
        expect(whitelist_token_by_user.kind.ExecutionError).toMatch(/ERR_NOT_AN_OWNER/i);

        await near.call("whitelist_token", {
                tips_available: true,
                token_id: null,
                min_deposit: "100000000000000000000000",
                min_tip: "10000000000000000000000",
                withdraw_commission: nearAPI.utils.format.parseNearAmount(withdraw_near_commission.toString()),
                dex: null,
                swap_contract_id: null,
                swap_pool_id: null
            }
            , {account_id: admin});

        await near.call("whitelist_token", {
                tips_available: true,
                token_id: ft_contract_account_id,
                min_deposit: "100000000000000000", // 0.1 DAI
                min_tip: "50000000000000000", // 0.05 DAI
                withdraw_commission: ConvertToPow18(withdraw_ft_commission),
                dex: "RefFinance",
                swap_contract_id: "ref-finance-101.testnet",
                swap_pool_id: 1669
            }
            , {account_id: admin});

        await near.call("whitelist_token", {
                tips_available: true,
                token_id: tiptoken_account_id,
                min_deposit: "100000000000000000", // 0.1 TT
                min_tip: "50000000000000000", // 0.05 TT
                withdraw_commission: ConvertToPow18(withdraw_ft_commission),
                dex: null,
                swap_contract_id: null,
                swap_pool_id: null,
            }
            , {account_id: admin});

        await near.call("whitelist_token", {
                tips_available: false,
                token_id: wrap_near_contract_id,
                min_deposit: "100000000000000000000000", // 0.1 wNEAR
                min_tip: "50000000000000000000000", // 0.05 wNEAR
                withdraw_commission: nearAPI.utils.format.parseNearAmount(withdraw_near_commission.toString()),
                dex: "RefFinance",
                swap_contract_id: "ref-finance-101.testnet",
                swap_pool_id: 1714,
            }
            , {account_id: admin});
    });

    test('Accounts has enough funds', async () => {
        const alice_wallet_balance = await near.accountNearBalance(alice);
        expect(alice_wallet_balance).toBeGreaterThan(20);

        const bob_wallet_balance = await near.accountNearBalance(alice);
        expect(bob_wallet_balance).toBeGreaterThan(20);
    });

    test('Register on REF', async () => {
        let ref = new contract("ref-finance-101.testnet");

        let ref_storage_deposit = await ref.call("storage_deposit", {}, {
            account_id: tipbot_account_id,
            tokens: utils.ConvertToNear(0.1)
        });
        expect(ref_storage_deposit.type).not.toBe('FunctionCallError');
    });
});

describe("Permissions", () => {
    test('Functions are unavailable', async () => {
        let func = await near.call("deposit_to_near_account", {
            account_id: alice,
            amount: tip_size
        }, {account_id: alice});
        expect(func.type).toBe('MethodNotFound');

        func = await near.call("set_deposit", {
            account_id: alice,
            token_account_id: ft_contract_account_id,
            amount: tip_size
        }, {account_id: alice});
        expect(func.type).toBe('MethodNotFound');

        func = await near.call("increase_deposit", {
            account_id: alice,
            token_account_id: ft_contract_account_id,
            amount: tip_size
        }, {account_id: alice});
        expect(func.type).toBe('MethodNotFound');

        func = await near.call("tip_from_account", {
            sender_account_id: alice,
            receiver_account_id: bob,
            contact: alice_contact_id,
            deposit: deposit_size
        }, {account_id: alice});
        expect(func.type).toBe('MethodNotFound');
    });

    let checkPrivateFunction = async function (method, args, result) {
        let func = await near.call(method, args, {account_id: alice});
        if (result && (func?.kind?.ExecutionError || func?.error?.type)) {
            expect(func?.kind?.ExecutionError || func?.error?.type).toMatch(result);
        } else {
            expect(func.type).toBe('MethodNotFound');
        }
    };

    test('Functions are private', async () => {
        await checkPrivateFunction("after_swap",
            {swap_contract_id: 1, account_id: alice, amount_in: "0", token_id: tiptoken_account_id, service_fee: "1"},
            "Method after_swap is private"
        );

        await checkPrivateFunction("on_withdraw",
            {account_id: alice, amount_withdraw: "0", token_id: tiptoken_account_id},
            "Method on_withdraw is private"
        );

        await checkPrivateFunction("internal_claim",
            {account_id: alice, token_id: tiptoken_account_id, amount: 1},
            "methodResolveError"
        );

        await checkPrivateFunction("update_config",
            {config: getConfig()},
            "ERR_NOT_AN_OWNER"
        );

        await checkPrivateFunction("set_withdraw_available",
            {withdraw_available: true},
            "ERR_NOT_AN_OWNER"
        );

        await checkPrivateFunction("set_tip_available",
            {tip_available: true},
            "ERR_NOT_AN_OWNER"
        );

        await checkPrivateFunction("increase_deposit",
            {account_id: alice, token_id: tiptoken_account_id, amount: 1},
            "methodResolveError"
        );

        await checkPrivateFunction("set_deposit",
            {account_id: alice, token_id: tiptoken_account_id, near_amount: 1},
            "methodResolveError"
        );

        await checkPrivateFunction("deposit_to_near_account",
            {account_id: alice, token_id: tiptoken_account_id, amount: 1, check_deposit_amount: true},
            "methodResolveError"
        );

        await checkPrivateFunction("after_ft_transfer_deposit",
            {account_id: alice, amount: "0", token_account_id: tiptoken_account_id},
            "Method after_ft_transfer_deposit is private"
        );

        await checkPrivateFunction("internal_withdraw_ft",
            {account_id: alice, token_account_id: tipbot_account_id, deposit: 1},
            "methodResolveError"
        );

        await checkPrivateFunction("withdraw_from_telegram",
            {telegram_account: bob_contact_id, account_id: alice, token_id: tipbot_account_id},
            "ERR_NO_ACCESS"
        );

        await checkPrivateFunction("transfer_tips_to_deposit",
            {telegram_account: bob_contact_id, account_id: alice, token_id: tipbot_account_id},
            "ERR_NO_ACCESS"
        );

        await checkPrivateFunction("migrate_import_accounts",
            {accounts: [], service_accounts: []},
            "ERR_NO_ACCESS"
        );

        await checkPrivateFunction("migration_transfer_tips_to_deposit",
            {telegram_account: bob_contact_id, account_id: alice, token_id: tipbot_account_id},
            "ERR_NO_ACCESS"
        );

        await checkPrivateFunction("insert_service_account_to_near_account",
            {account_id: bob, service_account: get_service_account(bob_contact_id)},
            "ERR_NO_ACCESS"
        );

        await checkPrivateFunction("remove_service_account_from_near_account",
            {account_id: bob, service_account: get_service_account(bob_contact_id)},
            "ERR_NO_ACCESS"
        );

        await checkPrivateFunction("internal_set_service_accounts_by_near_account",
            {}
        );

        await checkPrivateFunction("internal_tip",
            {},
            "methodResolveError"
        );

        await checkPrivateFunction("increase_balance",
            {},
            "methodResolveError"
        );


        await checkPrivateFunction("service_fees_add", {}, "methodResolveError");
        await checkPrivateFunction("service_fees_remove", {}, "methodResolveError");
        await checkPrivateFunction("treasury_add", {}, "methodResolveError");
        await checkPrivateFunction("treasury_remove", {}, "methodResolveError");
        await checkPrivateFunction("treasury_claimed_remove", {}, "methodResolveError");


        await checkPrivateFunction("increase_unclaimed_tips", {}, "methodResolveError");
        await checkPrivateFunction("set_unclaimed_tips_balance", {}, "methodResolveError");


        await checkPrivateFunction("whitelist_token",
            {  tips_available: true,
                token_id: tiptoken_account_id,
                min_deposit: "1",
                min_tip: "1",
                withdraw_commission: "1" },
            "ERR_NOT_AN_OWNER"
        );

        await checkPrivateFunction("transfer_unclaimed_tips_to_deposit",
            { service_account: get_service_account(bob_contact_id), token_id: tiptoken_account_id, receiver_account_id: bob},
            "ERR_NO_ACCESS"
        );


        await checkPrivateFunction("internal_withdraw", {}, "methodResolveError");
        await checkPrivateFunction("internal_withdraw_from_service_account", {}, "methodResolveError");

        await checkPrivateFunction("on_wrap_near",
            {account_id: alice, amount: "1"},
            "Method on_wrap_near is private"
        );

        await checkPrivateFunction("on_unwrap_near",
            {account_id: alice, amount: "1"},
            "Method on_unwrap_near is private"
        );









        /*
        let func = await near.call("after_ft_transfer_claim_by_chat", {
            chat_id: chat_id,
            amount_claimed: "1",
            token_account_id: ft_contract_account_id
        }, {account_id: alice});
        expect(func.type).toBe('FunctionCallError');

        func = await near.call("after_ft_transfer_deposit", {
            account_id: alice,
            amount: "1",
            token_account_id: ft_contract_account_id
        }, {account_id: alice});
        expect(func.type).toBe('FunctionCallError');

        func = await near.call("on_get_contact_owner_on_withdraw_from_telegram_with_auth", {
            account: alice,
            recipient_account_id: bob,
            contact: bob_contact_id
        }, {account_id: alice});
        expect(func.type).toBe('FunctionCallError');*/

    });

    test('Tip available', async () => {
        const tip_available_init = await near.call("set_tip_available", {tip_available: true}, {account_id: admin});
        const withdraw_available_init = await near.call("set_withdraw_available", {withdraw_available: true}, {account_id: admin});

        const deposit = await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        expect(deposit.type).not.toBe('FunctionCallError');

        const tip_unavailable = await near.call("set_tip_available", {tip_available: false}, {account_id: admin});
        expect(tip_unavailable.type).not.toBe('FunctionCallError');

        const send_tip_1 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });
        expect(send_tip_1.type).toBe('FunctionCallError');

        const tip_available = await near.call("set_tip_available", {tip_available: true}, {account_id: admin});
        expect(tip_available.type).not.toBe('FunctionCallError');

        const send_tip_2 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });
        expect(send_tip_2.type).not.toBe('FunctionCallError');

        const tip_unavailable_illegal = await near.call("set_tip_available", {tip_available: false}, {account_id: alice});
        expect(tip_unavailable_illegal.type).toBe('FunctionCallError');
    });

    test("Withdraw available", async () => {
        const withdraw_unavailable = await near.call("set_withdraw_available", {withdraw_available: false}, {account_id: admin});
        expect(withdraw_unavailable.type).not.toBe('FunctionCallError');

        const withdraw_1 = await near.call("withdraw", {}, {account_id: alice});
        expect(withdraw_1.type).toBe('FunctionCallError');

        const withdraw_available = await near.call("set_withdraw_available", {withdraw_available: true}, {account_id: admin});
        expect(withdraw_available.type).not.toBe('FunctionCallError');

        const withdraw_2 = await near.call("withdraw", {}, {account_id: alice});
        expect(withdraw_2.type).not.toBe('FunctionCallError');

        const withdraw_available_illegal = await near.call("set_withdraw_available", {withdraw_available: false}, {account_id: alice});
        expect(withdraw_available_illegal.type).toBe('FunctionCallError');
    });
});

describe("DepositWithdraw", () => {
    test('Deposit', async () => {
        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});

        const deposit = await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        expect(deposit.type).not.toBe('FunctionCallError');

        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});
        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBe(deposit_size);
    });

    test("Withdraw", async () => {
        const alice_deposit = await near.viewNearBalance("get_deposit", {account_id: alice});
        const alice_wallet_balance_1 = await near.accountNearBalance(alice);

        const withdraw = await near.call("withdraw", {}, {account_id: alice});
        expect(withdraw.type).not.toBe('FunctionCallError');

        const alice_deposit_3 = await near.viewNearBalance("get_deposit", {account_id: alice});
        expect(utils.RoundFloat(alice_deposit_3)).toBe(0);

        const alice_wallet_balance_2 = await near.accountNearBalance(alice);
        expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(alice_deposit, 1);
    });

    test('Deposit another account', async () => {
        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});
        const bob_wallet_balance_1 = await near.accountNearBalance(bob);

        const deposit = await near.call("deposit", {account_id: alice},
            {account_id: bob, tokens: utils.ConvertToNear(tip_size * 2)});
        expect(deposit.type).not.toBe('FunctionCallError');

        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});
        const bob_wallet_balance_2 = await near.accountNearBalance(bob);

        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBe(tip_size * 2);
        expect(utils.RoundFloat(bob_wallet_balance_1 - bob_wallet_balance_2)).toBeCloseTo(tip_size * 2, 1);
    });
});

describe("SimpleTip", () => {
    test("Test Deposit", async () => {
        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});
        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBe(deposit_size);
    });

    test("Test Tip", async () => {
        const bob_balance_1 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});

        const treasury_1 = await near.viewNearBalance("get_treasury", {token_id: null});
        const alice_treasury_1 = await near.viewNearBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: null
        });

        const send_tip_to_telegram_1 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });
        expect(send_tip_to_telegram_1.type).not.toBe('FunctionCallError');

        const bob_balance_2 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});
        expect(utils.RoundFloat(bob_balance_2 - bob_balance_1)).toBeCloseTo(tip_size - tip_size * treasury_fee, 5);

        const treasury_2 = await near.viewNearBalance("get_treasury", {token_id: null});
        expect(utils.RoundFloat(treasury_2 - treasury_1)).toBeCloseTo(tip_size * treasury_fee, 5);

        const alice_treasury_2 = await near.viewNearBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: null
        });
        expect(utils.RoundFloat(alice_treasury_2 - alice_treasury_1)).toBeCloseTo(tip_size * treasury_fee, 5);

        const send_tip_to_telegram_2 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size * 2),
        }, {
            account_id: alice
        });
        expect(send_tip_to_telegram_2.type).not.toBe('FunctionCallError');

        const bob_balance_3 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});
        expect(utils.RoundFloat(bob_balance_3 - bob_balance_2)).toBeCloseTo((tip_size - tip_size * treasury_fee) * 2, 5);

        const treasury_3 = await near.viewNearBalance("get_treasury", {token_id: null});
        expect(utils.RoundFloat(treasury_3 - treasury_2)).toBeCloseTo(tip_size * treasury_fee * 2, 5);
    });

    test("Test transfer_tips_to_deposit", async () => {
        const bob_balance = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});
        const bob_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: bob});

        const transfer_tips_to_deposit = await near.call("transfer_tips_to_deposit", {
            telegram_account: bob_contact_id,
            account_id: bob,
        }, {
            account_id: admin
        });
        expect(transfer_tips_to_deposit.type).not.toBe('FunctionCallError');

        const bob_balance_3 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});
        expect(utils.RoundFloat(bob_balance_3)).toBe(0);

        const bob_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: bob});

        expect(utils.RoundFloat(bob_deposit_2 - bob_deposit_1)).toBeCloseTo(bob_balance - withdraw_near_commission, 5);
    });
});

/*
describe("Tip with referral chat_id", () => {
    test("Tip with chat_id", async () => {
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        await near.call("deposit", {}, {account_id: bob, tokens: utils.ConvertToNear(deposit_size)});

        const add_chat_settings = await near.call("add_chat_settings", {
            chat_id: alice_chat_id,
            admin_account_id: chat_admin,
            treasure_fee_numerator,
            track_chat_points: true
        }, {account_id: admin});
        expect(add_chat_settings.type).not.toBe('FunctionCallError');

        const chat_score_1 = await near.view("get_chat_points", {chat_id: alice_chat_id});
        expect(chat_score_1).toBe(0);

        const send_tip_to_telegram_1 = await near.call("send_tip_to_telegram", {
            telegram_account: alice_contact_id,
            amount: utils.ConvertToNear(tip_size),
            chat_id: alice_chat_id
        }, {
            account_id: alice
        });
        expect(send_tip_to_telegram_1.type).not.toBe('FunctionCallError');

        const chat_score_2 = await near.view("get_chat_points", {chat_id: alice_chat_id});

        expect(chat_score_2 - chat_score_1).toBe(1);

        const send_tip_to_telegram_2_too_small = await near.call("send_tip_to_telegram", {
            telegram_account: alice_contact_id,
            amount: utils.ConvertToNear(0.00001),
            chat_id: alice_chat_id
        }, {
            account_id: alice
        });
        expect(send_tip_to_telegram_2_too_small.type).not.toBe('FunctionCallError');

        const chat_score_3 = await near.view("get_chat_points", {chat_id: alice_chat_id});
        expect(chat_score_3).toBe(chat_score_2);

        const send_tip_to_telegram_3 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
            chat_id: alice_chat_id
        }, {
            account_id: bob
        });
        expect(send_tip_to_telegram_3.type).not.toBe('FunctionCallError');

        const chat_score_4 = await near.view("get_chat_points", {chat_id: alice_chat_id});
        expect(chat_score_4 - chat_score_1).toBe(2);

        const send_tip_to_telegram_again = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
            chat_id: alice_chat_id
        }, {
            account_id: bob
        });
        expect(send_tip_to_telegram_again.type).not.toBe('FunctionCallError');

        const chat_score_5 = await near.view("get_chat_points", {chat_id: alice_chat_id});
        expect(chat_score_4).toBe(chat_score_5);
    });

});

 */

describe("ServiceTip", () => {
    test("Withdraw from telegram", async () => {
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});

        const remove_service_account = await near.call("remove_service_account_from_near_account", {
            account_id: carol,
            service_account: get_service_account(bob_contact_id)
        }, {
            account_id: admin
        });
        const remove_service_account_2 = await near.call("remove_service_account_from_near_account", {
            account_id: bob,
            service_account: get_service_account(carol_contact_id)
        }, {
            account_id: admin
        });

        let withdraw_init = await near.call("withdraw_from_telegram", {
            telegram_account: bob_contact_id,
            account_id: bob,
        }, {
            account_id: admin
        });

        const withdraw_init_2 = await near.call("withdraw", {}, {account_id: carol});

        const send_tip_to_telegram = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });
        expect(send_tip_to_telegram.type).not.toBe('FunctionCallError');

        const bob_balance = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});
        const bob_deposit = await near.viewNearBalance("get_deposit", {account_id: bob});
        const bob_wallet_balance_1 = await near.accountNearBalance(bob, 1000);

        let withdraw_1 = await near.call("withdraw_from_telegram", {
            telegram_account: bob_contact_id,
            account_id: bob,
        }, {
            account_id: admin
        });
        expect(withdraw_1.type).not.toBe('FunctionCallError');

        const bob_wallet_balance_2 = await near.accountNearBalance(bob, 1000);

        expect(utils.RoundFloat(bob_wallet_balance_2 - bob_wallet_balance_1)).toBeCloseTo(bob_deposit + bob_balance - withdraw_near_commission, 5);

        const bob_wallet_balance_3 = await near.accountNearBalance(bob, 1000);
        expect(utils.RoundFloat(bob_wallet_balance_3 - bob_wallet_balance_2)).toBeCloseTo(0, 1);

        let withdraw_3_no_funds = await near.call("withdraw_from_telegram", {
            telegram_account: bob_contact_id,
            account_id: bob,
            hint: "Withdraw from telegram-4"
        }, {
            account_id: admin
        });
        expect(withdraw_3_no_funds.type).toBe('FunctionCallError');
        expect(withdraw_3_no_funds.kind.ExecutionError).toMatch(/ERR_NOT_ENOUGH_TOKENS_TO_PAY_TRANSFER_COMMISSION/i);


        const bob_wallet_balance_4 = await near.accountNearBalance(bob, 1000);
        const bob_balance_2 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});
        const carol_wallet_balance_1 = await near.accountNearBalance(carol, 1000);

        const add_service_account = await near.call("insert_service_account_to_near_account", {
            account_id: carol,
            service_account: get_service_account(bob_contact_id)
        }, {
            account_id: admin
        });
        expect(add_service_account.type).not.toBe('FunctionCallError');

        const add_service_account_existing = await near.call("insert_service_account_to_near_account", {
            account_id: bob,
            service_account: get_service_account(bob_contact_id)
        }, {
            account_id: admin
        });
        expect(add_service_account_existing.type).toBe('FunctionCallError');
        expect(add_service_account_existing.kind.ExecutionError).toMatch(/ERR_SERVICE_ACCOUNT_ALREADY_SET_BY_OTHER_USER/i);

        const add_service_account_another_account_for_same_type = await near.call("insert_service_account_to_near_account", {
            account_id: carol,
            service_account: get_service_account(carol_contact_id)
        }, {
            account_id: admin
        });
        expect(add_service_account_another_account_for_same_type.type).toBe('FunctionCallError');
        expect(add_service_account_another_account_for_same_type.kind.ExecutionError).toMatch(/ERR_THIS_SERVICE_ACCOUNT_TYPE_ALREADY_SET_FOR_CURRENT_USER/i);

        const send_tip_to_telegram_2 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });
        expect(send_tip_to_telegram_2.type).not.toBe('FunctionCallError');

        // tip belongs to near account, not service account after we added service account
        let withdraw_no_tips = await near.call("withdraw_from_telegram", {
            telegram_account: bob_contact_id,
            account_id: bob,
        }, {
            account_id: admin
        });
        expect(withdraw_no_tips.type).toBe('FunctionCallError');
        expect(withdraw_no_tips.kind.ExecutionError).toMatch(/ERR_NOT_ENOUGH_TOKENS_TO_PAY_TRANSFER_COMMISSION/i);

        const withdraw = await near.call("withdraw", {}, {account_id: carol});
        expect(withdraw.type).not.toBe('FunctionCallError');

        const bob_wallet_balance_5 = await near.accountNearBalance(bob, 1000);
        const bob_balance_3 = await near.viewNearBalance("get_balance", {telegram_account: bob_contact_id});
        const carol_wallet_balance_2 = await near.accountNearBalance(carol, 1000);

        expect(utils.RoundFloat(bob_wallet_balance_5 - bob_wallet_balance_4)).toBeCloseTo(0, 5);
        expect(utils.RoundFloat(bob_balance_3 - bob_balance_2)).toBeCloseTo(0, 5);
        // numDigits = 2 since Carol pays for signing withdraw tx
        expect(utils.RoundFloat(carol_wallet_balance_2 - carol_wallet_balance_1)).toBeCloseTo(tip_size - tip_size * treasury_fee, 2);

    });
});

describe("Deposit and Tip Too Much", () => {
    test("Tip throws an error after deposit and sending double tip", async () => {
        //deposit
        await near.call("withdraw", {}, {account_id: alice});
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});

        // send double tip
        const illegal = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(deposit_size * 2),
        }, {
            account_id: alice
        });

        expect(illegal.type).toBe('FunctionCallError');
        expect(illegal.kind.ExecutionError).toMatch(/Not enough tokens/i);
    });
});

describe("Withdraw or Transfer by not an Admin", () => {
    test("Fail on withdraw_from_telegram from user", async () => {
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});

        await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });

        const illegal_withdraw = await near.call("withdraw_from_telegram", {
            telegram_account: bob_contact_id,
            account_id: alice,
        }, {
            account_id: alice
        });

        expect(illegal_withdraw.type).toBe('FunctionCallError');
        expect(illegal_withdraw.kind.ExecutionError).toMatch(/(ERR_NO_ACCESS)/i);
    });

    test("Fail on transfer_tips_to_deposit from user", async () => {
        const illegal_transfer = await near.call("transfer_tips_to_deposit", {
            telegram_account: bob_contact_id,
            account_id: alice,
        }, {
            account_id: alice
        });

        expect(illegal_transfer.type).toBe('FunctionCallError');
        expect(illegal_transfer.kind.ExecutionError).toMatch(/(ERR_NO_ACCESS)/i);
    });
});

const ft = new contract(ft_contract_account_id);

const tt = new contract(tiptoken_account_id);
/* Deploy token
near create-account token.zavodil.testnet --masterAccount=zavodil.testnet --initialBalance=3
near deploy token.zavodil.testnet /var/www/html/nearspace.info/apps/fungible_token.wasm new '{"owner_id": "zavodil.testnet", "total_supply": "1000000000000000000000000", "metadata": {"spec": "ft-1.0.0", "name": "Zavodil Token", "symbol": "ZAV", "decimals": 18}}'

near create-account tiptoken.zavodil.testnet --masterAccount=zavodil.testnet --initialBalance=3
near deploy tiptoken.zavodil.testnet /var/www/html/nearspace.info/apps/fungible_token.wasm new '{"owner_id": "zavodil.testnet", "total_supply": "1000000000000000000000000000000000", "metadata": {"spec": "ft-1.0.0", "name": "TipToken", "symbol": "TIP", "decimals": 24}}'

near call token.zavodil.testnet storage_deposit '{}' --accountId dev-1631555707974-69889981860912 --deposit 0.2
near call tiptoken.zavodil.testnet storage_deposit '{}' --accountId dev-1631555707974-69889981860912 --deposit 0.2
near call dev-1631555707974-69889981860912 whitelist_token '{"token_id": "near"}' --accountId zavodil.testnet
near call dev-1631555707974-69889981860912 whitelist_token '{"token_id": "token.zavodil.testnet"}' --accountId zavodil.testnet
near call dev-1631555707974-69889981860912 whitelist_token '{"token_id": "tiptoken.zavodil.testnet"}' --accountId zavodil.testnet


near call tiptoken.zavodil.testnet storage_deposit '{}' --accountId place.testnet --deposit 0.2
near call tiptoken.zavodil.testnet ft_transfer '{"receiver_id": "dev-1631555707974-69889981860912", "amount": "1000000000000000000000000000000"}' --accountId zavodil.testnet --amount 0.000000000000000000000001

Transfer
near call token.zavodil.testnet storage_deposit '{}' --accountId grant.testnet --deposit 0.1
near call token.zavodil.testnet storage_deposit '{}' --accountId place.testnet --deposit 0.1
near call token.zavodil.testnet ft_transfer '{"receiver_id": "grant.testnet", "amount": "1000000000000000000000"}' --accountId zavodil.testnet --amount 0.000000000000000000000001
near call token.zavodil.testnet ft_transfer '{"receiver_id": "place.testnet", "amount": "1000000000000000000000"}' --accountId zavodil.testnet --amount 0.000000000000000000000001

near view token.zavodil.testnet ft_balance_of '{"account_id": "grant.testnet"}'
 */

describe("Tip FT", () => {
    test("Fail on not whitelisted token", async () => {
        const illegal_token = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: ft_tip_size,
            token_id: "testnet"
        }, {
            account_id: alice
        });
        expect(illegal_token.type).toBe('FunctionCallError');
        expect(illegal_token.kind.ExecutionError).toMatch(/(ERR_TOKEN_WAS_NOT_WHITELISTED)/i);

        /*
        const whitelist_token = await near.call("whitelist_token", {
            token_id: ft_contract_account_id,
        }, {
            account_id: admin
        });
        expect(whitelist_token.type).not.toBe('FunctionCallError');*/
    });

    test("Deposit FT", async () => {
        const alice_deposit_1 = await near.view("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});

        const deposit = await ft.call("ft_transfer_call",
            {receiver_id: tipbot_account_id, amount: ft_deposit_size, msg: ""},
            {account_id: alice, tokens: 1});
        expect(deposit.type).not.toBe('FunctionCallError');

        const alice_deposit_2 = await near.view("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});
        expect(utils.RoundFloat(alice_deposit_2) - utils.RoundFloat(alice_deposit_1)).toBe(utils.RoundFloat(ft_deposit_size));
    });

    test("Send FT tips", async () => {
        const remove_service_account = await near.call("remove_service_account_from_near_account", {
            account_id: carol,
            service_account: get_service_account(bob_contact_id)
        }, {
            account_id: admin
        });

        const bob_tips_1 = await near.viewDaiBalance("get_balance",
            {telegram_account: bob_contact_id, token_id: ft_contract_account_id});
        const alice_deposit_1 = await near.viewDaiBalance("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});

        const sent_tip = await near.call("send_tip_to_telegram",
            {telegram_account: bob_contact_id, amount: ft_tip_size, token_id: ft_contract_account_id},
            {account_id: alice});
        expect(sent_tip.type).not.toBe('FunctionCallError');

        const bob_tips_2 = await near.viewDaiBalance("get_balance",
            {telegram_account: bob_contact_id, token_id: ft_contract_account_id});
        const alice_deposit_2 = await near.viewDaiBalance("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});

        expect(utils.RoundFloat(alice_deposit_1 - alice_deposit_2)).toBe(utils.ConvertFromDai(ft_tip_size));
        expect(utils.RoundFloat(bob_tips_2 - bob_tips_1)).toBeCloseTo(utils.ConvertFromDai(ft_tip_size - ft_tip_size * treasury_fee), 3);

        const sent_tip_2 = await near.call("send_tip_to_telegram",
            {telegram_account: bob_contact_id, amount: ft_tip_size, token_id: ft_contract_account_id},
            {account_id: alice});
        expect(sent_tip_2.type).not.toBe('FunctionCallError');

        const bob_tips_3 = await near.viewDaiBalance("get_balance",
            {telegram_account: bob_contact_id, token_id: ft_contract_account_id});
        const alice_deposit_3 = await near.viewDaiBalance("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});

        expect(utils.RoundFloat(alice_deposit_1 - alice_deposit_3)).toBe(utils.ConvertFromDai(ft_tip_size) * 2);
        expect(utils.RoundFloat(bob_tips_3 - bob_tips_1)).toBeCloseTo(utils.ConvertFromDai(ft_tip_size - ft_tip_size * treasury_fee) * 2, 3);
    });

    test("Withdraw FT tips from balance", async () => {
        const sent_tip = await near.call("send_tip_to_telegram",
            {telegram_account: bob_contact_id, amount: ft_tip_size, token_id: ft_contract_account_id},
            {account_id: alice});
        expect(sent_tip.type).not.toBe('FunctionCallError');

        const bob_ft_balance_1 = await ft.viewDaiBalance("ft_balance_of", {account_id: bob});

        const bob_tips_1 = await near.viewDaiBalance("get_balance",
            {telegram_account: bob_contact_id, token_id: ft_contract_account_id});
        expect(utils.RoundFloat(bob_tips_1)).toBeGreaterThan(0);

        const withdraw = await near.call("withdraw_from_telegram",
            {telegram_account: bob_contact_id, account_id: bob, token_id: ft_contract_account_id},
            {account_id: admin});
        expect(withdraw.type).not.toBe('FunctionCallError');

        const bob_tips_2 = await near.viewDaiBalance("get_balance",
            {telegram_account: bob_contact_id, token_id: ft_contract_account_id});

        const bob_ft_balance_2 = await ft.viewDaiBalance("ft_balance_of", {account_id: bob});

        expect(utils.RoundFloat(bob_tips_2)).toBe(0);
        expect(utils.RoundFloat(bob_ft_balance_2 - bob_ft_balance_1)).toBeCloseTo(bob_tips_1 - withdraw_ft_commission, 3);
    });

    test("Withdraw FT tips from deposit", async () => {
        const withdraw_at_start = await near.call("withdraw",
            {token_id: ft_contract_account_id},
            {account_id: alice});

        const alice_ft_balance_1 = await ft.viewDaiBalance("ft_balance_of", {account_id: alice});

        const alice_ft_deposit_1 = await near.viewDaiBalance("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});

        const deposit = await ft.call("ft_transfer_call",
            {receiver_id: tipbot_account_id, amount: ft_deposit_size, msg: ""},
            {account_id: alice, tokens: 1});
        expect(deposit.type).not.toBe('FunctionCallError');

        const alice_ft_deposit_2 = await near.viewDaiBalance("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});
        expect(utils.RoundFloat(alice_ft_deposit_2 - alice_ft_deposit_1)).toBe(utils.ConvertFromDai(ft_deposit_size));

        const alice_ft_balance_2 = await ft.viewDaiBalance("ft_balance_of", {account_id: alice});
        expect(utils.RoundFloat(alice_ft_balance_1 - alice_ft_balance_2)).toBe(utils.ConvertFromDai(ft_deposit_size));

        const withdraw = await near.call("withdraw",
            {token_id: ft_contract_account_id},
            {account_id: alice});
        expect(withdraw.type).not.toBe('FunctionCallError');

        const alice_ft_deposit_3 = await near.viewDaiBalance("get_deposit",
            {account_id: alice, token_id: ft_contract_account_id});

        expect(utils.RoundFloat(alice_ft_deposit_3)).toBe(0);

        const alice_ft_balance_3 = await ft.viewDaiBalance("ft_balance_of", {account_id: alice});
        expect(utils.RoundFloat(alice_ft_balance_3)).toBe(alice_ft_balance_1);
        expect(utils.RoundFloat(alice_ft_balance_3 - alice_ft_balance_2)).toBe(utils.ConvertFromDai(ft_deposit_size));

        const withdraw_2_no_funds = await near.call("withdraw",
            {token_id: ft_contract_account_id},
            {account_id: alice});
        expect(withdraw_2_no_funds.type).toBe('FunctionCallError');
        expect(withdraw_2_no_funds.kind.ExecutionError).toMatch(/ERR_ZERO_BALANCE/i);
    });

    test("Service Fee generation for FT", async () => {
        const deposit = await ft.call("ft_transfer_call",
            {receiver_id: tipbot_account_id, amount: ft_deposit_size, msg: ""},
            {account_id: alice, tokens: 1});
        expect(deposit.type).not.toBe('FunctionCallError');

        const treasury_1 = await near.viewDaiBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: ft_contract_account_id
        });

        const sent_tip = await near.call("send_tip_to_telegram",
            {telegram_account: bob_contact_id, amount: ft_tip_size, token_id: ft_contract_account_id},
            {account_id: alice});
        expect(sent_tip.type).not.toBe('FunctionCallError');

        const treasury_2 = await near.viewDaiBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: ft_contract_account_id
        });

        expect(utils.RoundFloat(treasury_2 - treasury_1)).toBeCloseTo(utils.RoundFloat(utils.ConvertFromDai(ft_tip_size) * treasury_fee), 3);

        const service_fee_1 = await near.viewDaiBalance("get_service_fee", {token_id: ft_contract_account_id});

        const claim_all_ft = await near.call("claim_tiptoken",
            {token_id: ft_contract_account_id},
            {account_id: alice, gas: 200000000000000});
        expect(claim_all_ft.type).not.toBe('FunctionCallError');

        const service_fee_2 = await near.viewDaiBalance("get_service_fee", {token_id: ft_contract_account_id});

        expect(utils.RoundFloat(service_fee_2 - service_fee_1)).toBeCloseTo(utils.RoundFloat(utils.ConvertFromDai(ft_tip_size) * treasury_fee * treasury_fee), 3);
    });

    test("Service Fee generation for TT", async () => {
        const tt_init_tokens = await tt.call("ft_transfer_call",
            {receiver_id: alice, amount: ft_deposit_size, msg: ""},
            {account_id: tiptoken_owner_account_id, tokens: 1});
        expect(tt_init_tokens.type).not.toBe('FunctionCallError');

        const deposit = await tt.call("ft_transfer_call",
            {receiver_id: tipbot_account_id, amount: ft_deposit_size, msg: ""},
            {account_id: alice, tokens: 1});
        expect(deposit.type).not.toBe('FunctionCallError');

        const treasury_1 = await near.viewDaiBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: tiptoken_account_id
        });

        const sent_tip = await near.call("send_tip_to_telegram",
            {telegram_account: bob_contact_id, amount: ft_tip_size, token_id: tiptoken_account_id},
            {account_id: alice});
        expect(sent_tip.type).not.toBe('FunctionCallError');

        const treasury_2 = await near.viewDaiBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: tiptoken_account_id
        });
        expect(utils.RoundFloat(treasury_2 - treasury_1)).toBe(utils.RoundFloat(0));


        const service_fee_1 = await near.viewDaiBalance("get_service_fee", {token_id: tiptoken_account_id});

        const claim_tiptoken_illegal = await near.call("claim_tiptoken",
            {token_id: tiptoken_account_id},
            {account_id: alice, gas: 200000000000000});
        expect(claim_tiptoken_illegal.type).toBe('FunctionCallError');

        const service_fee_2 = await near.viewDaiBalance("get_service_fee", {token_id: tiptoken_account_id});

        expect(utils.RoundFloat(service_fee_2 - service_fee_1)).toBe(utils.RoundFloat(0));
    });
});

describe("Claim TipToken", () => {

    test("Claim TipToken for FT", async () => {
        const deposit = await ft.call("ft_transfer_call",
            {receiver_id: tipbot_account_id, amount: ft_deposit_size, msg: ""},
            {account_id: alice, tokens: 1});
        expect(deposit.type).not.toBe('FunctionCallError');

        const bot_tt_balance_1 = await tt.viewDaiBalance("ft_balance_of", {account_id: tipbot_account_id});
        const alice_ft_treasury_1 = await near.viewDaiBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: ft_contract_account_id
        });
        const alice_tt_deposit_1 = await near.viewDaiBalance("get_deposit", {
            account_id: alice,
            token_id: tiptoken_account_id
        });
        const service_fee_1 = await near.viewDaiBalance("get_service_fee", {token_id: ft_contract_account_id});

        const sent_tip = await near.call("send_tip_to_telegram",
            {telegram_account: bob_contact_id, amount: ft_tip_size, token_id: ft_contract_account_id},
            {account_id: alice});
        expect(sent_tip.type).not.toBe('FunctionCallError');

        const alice_ft_treasury_2 = await near.viewDaiBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: ft_contract_account_id
        });
        expect(utils.RoundFloat(alice_ft_treasury_2 - alice_ft_treasury_1)).toBeCloseTo(utils.ConvertFromDai(ft_tip_size) * treasury_fee, 3);

        const claim_all_tiptoken = await near.call("claim_tiptoken",
            {token_id: ft_contract_account_id},
            {account_id: alice, gas: 200000000000000});
        expect(claim_all_tiptoken.type).not.toBe('FunctionCallError');

        // no more tt to claim
        const alice_ft_treasury_3 = await near.viewDaiBalance("get_unclaimed_treasury", {
            account_id: alice,
            token_id: ft_contract_account_id
        }, {}, 1000);
        expect(utils.RoundFloat(alice_ft_treasury_3)).toBeCloseTo(0, 5);

        // alice got tt
        const alice_tt_deposit_2 = await near.viewDaiBalance("get_deposit", {
            account_id: alice,
            token_id: tiptoken_account_id
        });
        expect(utils.RoundFloat(alice_tt_deposit_2)).toBeGreaterThan(utils.RoundFloat(alice_tt_deposit_1));

        // service_fee earned
        const service_fee_2 = await near.viewDaiBalance("get_service_fee", {token_id: ft_contract_account_id});

        expect(utils.RoundFloat(service_fee_2 - service_fee_1)).toBeGreaterThan(0);
        expect(utils.RoundFloat(service_fee_2 - service_fee_1)).toBeCloseTo(utils.RoundFloat((alice_ft_treasury_2 - alice_ft_treasury_1) * service_fee));

        // bot got tt
        const bot_tt_balance_2 = await tt.viewDaiBalance("ft_balance_of", {account_id: tipbot_account_id});
        expect(utils.RoundFloat(bot_tt_balance_2)).toBeGreaterThan(utils.RoundFloat(bot_tt_balance_1));
        expect(utils.RoundFloat(bot_tt_balance_2 - bot_tt_balance_1)).toBeCloseTo(utils.RoundFloat(alice_tt_deposit_2 - alice_tt_deposit_1), 3);

        const claim_all_tiptoken_without_funds = await near.call("claim_tiptoken",
            {token_id: ft_contract_account_id},
            {account_id: alice, gas: 200000000000000});
        expect(claim_all_tiptoken_without_funds.type).toBe('FunctionCallError');
        expect(claim_all_tiptoken_without_funds.kind.ExecutionError).toMatch(/ERR_AMOUNT_IS_NOT_POSITIVE/i);

        // no more tt for alice
        const alice_tt_deposit_3 = await near.viewDaiBalance("get_deposit", {
            account_id: alice,
            token_id: tiptoken_account_id
        });
        expect(utils.RoundFloat(alice_tt_deposit_3)).toBe(utils.RoundFloat(alice_tt_deposit_2));


        // withdraw TT
        const alice_tt_balance_1 = await tt.viewDaiBalance("ft_balance_of", {account_id: alice});

        const withdraw = await near.call("withdraw",
            {token_id: tiptoken_account_id},
            {account_id: alice});
        expect(withdraw.type).not.toBe('FunctionCallError');

        const alice_tt_balance_2 = await tt.viewDaiBalance("ft_balance_of", {account_id: alice});
        expect(utils.RoundFloat(alice_tt_balance_2 - alice_tt_balance_1)).toBe(utils.RoundFloat(alice_tt_deposit_2));

        // no more tt for alice's bot account
        const alice_tt_deposit_4 = await near.viewDaiBalance("get_deposit", {
            account_id: alice,
            token_id: tiptoken_account_id
        });
        expect(utils.RoundFloat(alice_tt_deposit_4)).toBe(0);
    });
});

describe("Chat Points", () => {
    /*
    test("Send NEAR tips with chat_id", async () => {
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});

        const random_telegram_id_1 = Date.now();

        const chat_score_1 = await near.viewNearBalance("get_chat_tokens", {chat_id: chat_id});
        const sender_score_1 = await near.viewNearBalance("get_user_tokens", {account_id: alice, chat_id: chat_id});

        const treasure_balance_1 = await near.viewNearBalance("get_treasure_balance", {});

        let send_1 = await near.call("send_tip_to_telegram", {
            telegram_account: random_telegram_id_1,
            amount: utils.ConvertToNear(tip_size),
            chat_id: chat_id
        }, {
            account_id: alice
        });
        expect(send_1.type).not.toBe('FunctionCallError');

        const chat_score_2 = await near.viewNearBalance("get_chat_tokens", {chat_id: chat_id});
        expect(utils.RoundFloat(chat_score_2 - chat_score_1)).toBeCloseTo(tip_size * treasure_fee_numerator * chat_tiptoken_fraction / 100, 3);

        const sender_score_2 = await near.viewNearBalance("get_user_tokens", {account_id: alice, chat_id: chat_id});
        expect(utils.RoundFloat(sender_score_2 - sender_score_1)).toBeCloseTo(tip_size * treasure_fee_numerator * sender_tiptoken_fraction / 100, 3);

        const treasure_balance_2 = await near.viewNearBalance("get_treasure_balance", {});
        expect(utils.RoundFloat(treasure_balance_2 - treasure_balance_1)).toBe(tip_size * treasure_fee_numerator / 100);

        let send_2 = await near.call("send_tip_to_telegram", {
            telegram_account: random_telegram_id_1,
            amount: utils.ConvertToNear(tip_size),
            chat_id: chat_id
        }, {
            account_id: alice
        });
        expect(send_2.type).not.toBe('FunctionCallError');

        const chat_score_3 = await near.viewNearBalance("get_chat_tokens", {chat_id: chat_id});
        expect(utils.RoundFloat(chat_score_3 - chat_score_2)).toBeCloseTo(tip_size * treasure_fee_numerator * chat_tiptoken_fraction / 100, 3);

        const sender_score_3 = await near.viewNearBalance("get_user_tokens", {account_id: alice, chat_id: chat_id});
        expect(utils.RoundFloat(sender_score_3 - sender_score_2)).toBeCloseTo(tip_size * treasure_fee_numerator  * sender_tiptoken_fraction / 100, 3);

        const treasure_balance_3 = await near.viewNearBalance("get_treasure_balance", {});
        expect(utils.RoundFloat(treasure_balance_3 - treasure_balance_2)).toBe(tip_size * treasure_fee_numerator / 100);
    });

    test("Claim TipToken", async () => {
        const chat_score_1 = await near.viewNearBalance("get_chat_tokens", {chat_id: chat_id});
        const sender_score_1 = await near.viewNearBalance("get_user_tokens", {account_id: alice, chat_id: chat_id});
        const treasure_balance_1 = await near.viewNearBalance("get_treasure_balance", {});

        const tiptoken_balance_chat_admin_1 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: chat_admin});
        const tiptoken_balance_alice_1 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: alice});

        const get_total_tiptokens_1 = await near.viewNearBalance("get_total_tiptokens", {});

        const claim_tiptokens_for_chat_wrong = await near.call("claim_tiptokens_for_chat", {chat_id}, {account_id: alice, tokens: 1});
        expect(claim_tiptokens_for_chat_wrong.type).toBe('FunctionCallError');
        expect(claim_tiptokens_for_chat_wrong.kind.ExecutionError).toMatch(/(Current user is not a chat admin)/i);

        const claim_tiptokens_for_chat = await near.call("claim_tiptokens_for_chat", {chat_id}, {account_id: chat_admin, tokens: 1});
        expect(claim_tiptokens_for_chat.type).not.toBe('FunctionCallError');

        const tiptoken_balance_chat_admin_2 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: chat_admin});
        expect(utils.RoundFloat(tiptoken_balance_chat_admin_2 - tiptoken_balance_chat_admin_1)).toBeCloseTo(utils.RoundFloat(chat_score_1), 3);

        const claim_tiptokens_for_user = await near.call("claim_tiptokens", {}, {account_id: alice, tokens: 1});
        expect(claim_tiptokens_for_user.type).not.toBe('FunctionCallError');

        const tiptoken_balance_alice_2 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: alice});
        expect(utils.RoundFloat(tiptoken_balance_alice_2 - tiptoken_balance_alice_1)).toBeCloseTo(utils.RoundFloat(sender_score_1), 3);

        const chat_score_2 = await near.viewNearBalance("get_chat_tokens", {chat_id: chat_id});
        const sender_score_2 = await near.viewNearBalance("get_user_tokens", {account_id: alice, chat_id: chat_id});
        const treasure_balance_2 = await near.viewNearBalance("get_treasure_balance", {});

        const get_total_tiptokens_2 = await near.viewNearBalance("get_total_tiptokens", {});
        expect(utils.RoundFloat(get_total_tiptokens_2 - get_total_tiptokens_1)).toBeCloseTo(utils.RoundFloat(chat_score_1 + sender_score_1), 3);

        expect(utils.RoundFloat(chat_score_2)).toBe(0);
        expect(utils.RoundFloat(sender_score_2)).toBe(0);
        expect(treasure_balance_1).toBe(treasure_balance_2);
    });

    test("Redeem Tiptokens", async () => {

        const tiptoken_balance_alice_1 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: alice});
        const tiptoken_balance_app_1 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: tipbot_account_id});
        const get_total_tiptokens_1 = await near.viewNearBalance("get_total_tiptokens", {});

         const add_chat_settings = await near.call("add_chat_settings", {
            chat_id: chat_id,
            admin_account_id: chat_admin,
            treasure_fee_numerator,
            track_chat_points: true
        }, {account_id: admin});
        expect(add_chat_settings.type).not.toBe('FunctionCallError');

        const chat_settings = await near.view("get_chat_settings", {chat_id: chat_id});
        expect(chat_settings.admin_account_id).toBe(chat_admin);

        const deposit = await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        expect(deposit.type).not.toBe('FunctionCallError');

        let send_near = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: tip_size,
            chat_id: chat_id,
        }, {
            account_id: alice
        });
        expect(send_near.type).not.toBe('FunctionCallError');

        const get_total_tiptokens_2 = await near.viewNearBalance("get_total_tiptokens", {});
        expect(utils.RoundFloat(get_total_tiptokens_2 - get_total_tiptokens_1)).toBeCloseTo(utils.RoundFloat(tip_size * (chat_tiptoken_fraction + sender_tiptoken_fraction)), 3);


        // ft_tansfer
        // redeem
        // check balance
    });

 */

    /*
    test("Send FT tips with chat_id", async () => {
        const deposit = await ft.call("ft_transfer_call",
            {receiver_id: tipbot_account_id, amount: ft_deposit_size, msg: ""},
            {account_id: alice, tokens: 1});
        expect(deposit.type).not.toBe('FunctionCallError');

        const random_telegram_id_1 = Date.now();

        const chat_score_1 = await near.viewDaiBalance("get_chat_tokens", {
            chat_id: chat_id,
            token_id: ft_contract_account_id
        });

        const treasure_ft_balance_1 = await near.viewDaiBalance("get_treasure_balance", {token_account_id: ft_contract_account_id});

        let send_1 = await near.call("send_tip_to_telegram", {
            telegram_account: random_telegram_id_1,
            amount: ft_tip_size,
            chat_id: chat_id,
            token_id: ft_contract_account_id
        }, {
            account_id: alice
        });
        expect(send_1.type).not.toBe('FunctionCallError');

        const chat_score_2 = await near.viewDaiBalance("get_chat_tokens", {
            chat_id: chat_id, token_id: ft_contract_account_id
        });
        expect(utils.RoundFloat(chat_score_2 - chat_score_1)).toBeCloseTo(utils.ConvertFromDai(ft_tip_size) * treasure_fee_numerator / 100 * chat_tiptoken_fraction, 3);

        const treasure_ft_balance_2 = await near.viewDaiBalance("get_treasure_balance", {token_account_id: ft_contract_account_id});
        expect(utils.RoundFloat(treasure_ft_balance_2 - treasure_ft_balance_1)).toBeCloseTo(utils.ConvertFromDai(ft_tip_size * treasure_fee_numerator / 100), 3);

        let send_2 = await near.call("send_tip_to_telegram", {
            telegram_account: random_telegram_id_1,
            amount: ft_tip_size,
            chat_id: chat_id,
            token_id: ft_contract_account_id
        }, {
            account_id: alice
        });
        expect(send_2.type).not.toBe('FunctionCallError');

        const chat_score_3 = await near.viewDaiBalance("get_chat_tokens", {
            chat_id: chat_id, token_id: ft_contract_account_id
        });
        expect(utils.RoundFloat(chat_score_3 - chat_score_2)).toBeCloseTo(utils.ConvertFromDai(ft_tip_size) * treasure_fee_numerator / 100 * chat_tiptoken_fraction, 3);

        const treasure_ft_balance_3 = await near.viewDaiBalance("get_treasure_balance", {token_account_id: ft_contract_account_id});
        expect(utils.RoundFloat(treasure_ft_balance_3 - treasure_ft_balance_2)).toBeCloseTo(utils.ConvertFromDai(ft_tip_size * treasure_fee_numerator / 100), 3);
    });
     */

    /*
    test("Chat Tokens", async () => {
        const add_chat_settings = await near.call("add_chat_settings", {
            chat_id: chat_id,
            admin_account_id: chat_admin,
            treasure_fee_numerator,
            track_chat_points: true
        }, {account_id: admin});
        expect(add_chat_settings.type).not.toBe('FunctionCallError');

        const chat_settings = await near.view("get_chat_settings", {chat_id: chat_id});
        expect(chat_settings.admin_account_id).toBe(chat_admin);

        let claim_chat_tokens_0 = await near.call("claim_tiptokens_for_chat", {
            chat_id: chat_id,
        }, {
            account_id: chat_admin
        });

        const chat_score_0 = await near.viewNearBalance("get_chat_tokens", {
            chat_id: chat_id,
        });
        expect(utils.RoundFloat(chat_score_0)).toBe(0);

        const app_balance_0 = await near.accountNearBalance(tipbot_account_id);
        const chat_admin_balance_0 = await near.accountNearBalance(chat_admin);
        const sender_balance_0 = await near.accountNearBalance(alice);

        const deposit = await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        expect(deposit.type).not.toBe('FunctionCallError');

        let send_near = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact_id,
            amount: ft_tip_size,
            chat_id: chat_id,
        }, {
            account_id: alice
        });
        expect(send_near.type).not.toBe('FunctionCallError');

        const app_balance_1 = await near.accountNearBalance(tipbot_account_id);
        expect(utils.RoundFloat(app_balance_1 - app_balance_0)).toBe(0);

        const chat_admin_balance_1 = await near.accountNearBalance(chat_admin);
        expect(utils.RoundFloat(chat_admin_balance_1 - chat_admin_balance_0)).toBe(0);

        const sender_balance_1 = await near.accountNearBalance(alice);
        expect(utils.RoundFloat(sender_balance_1 - sender_balance_0)).toBe(0);

        const admin_deposit_1 = await near.accountNearBalance(chat_admin);
        const chat_score_1 = await near.viewNearBalance("get_chat_tokens", {
            chat_id: chat_id
        });

        const tiptoken_balance_chat_admin_1 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: chat_admin});

        let claim_chat_tokens_wrong = await near.call("claim_chat_tokens", {
            chat_id: chat_id,
        }, {
            account_id: alice
        });
        expect(claim_chat_tokens_wrong.kind.ExecutionError).toMatch(/(Current user is not a chat admin)/i);

        let claim_chat_tokens_1 = await near.call("claim_tiptokens_for_chat", {
            chat_id: chat_id,
        }, {
            account_id: chat_admin
        });
        expect(claim_chat_tokens_1.type).not.toBe('FunctionCallError');

        const tiptoken_balance_chat_admin_2 = await tiptoken_contract.viewNearBalance("ft_balance_of", {account_id: chat_admin});

        const chat_ft_balance_2 = await ft.viewDaiBalance("ft_balance_of", {account_id: tipbot_account_id});
        expect(utils.RoundFloat(chat_ft_balance_1 - chat_ft_balance_2)).toBe(utils.ConvertFromDai(ft_tip_size / 100 * treasure_fee_numerator));

        const chat_admin_score_2 = await ft.viewDaiBalance("ft_balance_of", {account_id: chat_admin});
        expect(utils.RoundFloat(chat_admin_score_2 - chat_admin_score_1)).toBe(utils.ConvertFromDai(ft_tip_size / 100 * treasure_fee_numerator));

        const admin_deposit_2 = await near.accountNearBalance(chat_admin);
        const chat_score_2 = await near.viewNearBalance("get_chat_tokens", {
            chat_id: chat_id,
            token_id: ft_contract_account_id
        });
        expect(utils.RoundFloat(admin_deposit_2 - admin_deposit_1)).toBeCloseTo(chat_score_1, 1);
        expect(utils.RoundFloat(chat_score_2)).toBe(0);


        let claim_chat_tokens_2_no_funds = await near.call("claim_chat_tokens", {
            chat_id: chat_id,
            token_id: ft_contract_account_id
        }, {
            account_id: chat_admin
        });
        expect(claim_chat_tokens_2_no_funds.type).toBe('FunctionCallError');
        expect(claim_chat_tokens_2_no_funds.kind.ExecutionError).toMatch(/(Nothing to claim)/i);
    });
*/
});

const auth = new contract("dev-1625611642901-32969379055293");
const request_key = "63b2f81544f5ee526191c3f8b8fcccf8c8e7d689c0407ddd1fb91f20e66ca04c";
const request_secret = "5bSXPcb1D4BT7KQWHxpMDpv8wvgyvcrbwYH55AcctZtpb1Vc6QQFgsL6evJ7HxuW2SrusPuDurHctELgr4X9JQwj";
const WHITELIST_FEE = 0.0015;

describe("Create contact and send tip", () => {
    test("Auth contract: " + auth.contract_id, async () => {
        expect(auth.contract_id).not.toBe(undefined)
    });

    test("Whitelist request key", async () => {
        const already_has_request = await auth.view("has_request_key", {account_id: alice}, {});
        if (already_has_request)
            await auth.call("remove_request", {}, {account_id: alice});

        const storage_deposit = await auth.call("storage_deposit", {}, {
            account_id: alice,
            tokens: utils.ConvertToNear(0.1)
        });
        expect(storage_deposit.type).not.toBe('FunctionCallError');

        const whitelist_key = await auth.call("whitelist_key", {
            account_id: alice,
            request_key: request_key,
        }, {
            account_id: admin
        });
        expect(whitelist_key.type).not.toBe('FunctionCallError');

        const whitelist_key_illegal = await auth.call("whitelist_key", {
            account_id: alice,
            request_key: request_key,
        }, {
            account_id: admin
        });
        expect(whitelist_key_illegal.type).toBe('FunctionCallError');

        const key = await auth.view("get_request_key", {account_id: alice}, {});
        expect(key).toBe(request_key);

        const has_key = await auth.view("has_request_key", {account_id: alice}, {});
        expect(has_key).toBeTruthy()
    });

    test("Create contact", async () => {
        const is_need_to_remove_contact = await auth.view("is_owner", {
            account_id: alice,
            contact: {
                category: "Telegram",
                value: alice_contact_handler,
                account_id: Number(alice_contact_id)
            }
        }, {});

        if (is_need_to_remove_contact) {
            const remove = await auth.call("remove", {
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: Number(alice_contact_id)
                }
            }, {account_id: alice});
            expect(remove.type).not.toBe('FunctionCallError');
        }

        const start_auth = await auth.call("start_auth", {
            request_key: request_key,
            contact: {
                category: "Telegram",
                value: alice_contact_handler,
                account_id: Number(alice_contact_id)
            }
        }, {
            account_id: alice,
            tokens: 1
        });

        expect(start_auth.type).not.toBe('FunctionCallError');

        const confirm_auth_illegal = await auth.call("confirm_auth", {
            key: request_secret,
        }, {account_id: bob});
        expect(confirm_auth_illegal.type).toBe('FunctionCallError');

        const confirm_auth = await auth.call("confirm_auth", {
            key: request_secret,
        }, {account_id: alice});
        expect(confirm_auth.type).not.toBe('FunctionCallError');

        const is_owner = await auth.view("is_owner", {
            account_id: alice,
            contact: {
                category: "Telegram",
                value: alice_contact_handler,
                account_id: Number(alice_contact_id)
            }
        }, {});
        expect(is_owner).toBeTruthy();

        const is_owner_without_value = await auth.view("is_owner", {
            account_id: alice,
            contact: {
                category: "Telegram",
                value: "",
                account_id: Number(alice_contact_id)
            }
        }, {});
        expect(is_owner_without_value).toBeTruthy();

        const is_owner_invalid = await auth.view("is_owner", {
            account_id: alice,
            contact: {
                category: "Telegram",
                value: bob_contact_handler,
                account_id: Number(bob_contact_id)
            }
        }, {});
        expect(is_owner_invalid).not.toBeTruthy();

        const get_account_for_contact_alice = await auth.view("get_account_for_contact", {
            contact: {
                category: "Telegram",
                value: alice_contact_handler,
                account_id: Number(alice_contact_id)
            }
        }, {});
        expect(get_account_for_contact_alice).toBe(alice);

        const get_account_for_contact_alice_without_value = await auth.view("get_account_for_contact", {
            contact: {
                category: "Telegram",
                value: "",
                account_id: Number(alice_contact_id)
            }
        }, {});
        expect(get_account_for_contact_alice_without_value).toBe(alice);

        const get_account_for_contact_bob = await auth.view("get_account_for_contact", {
            contact: {
                category: "Telegram",
                value: bob_contact_handler,
                account_id: Number(bob_contact_id)
            }
        }, {});
        expect(get_account_for_contact_bob).not.toBe(alice);

        const get_account_for_contact_alice_missing_account_id = await auth.view("get_account_for_contact", {
            contact: {
                category: "Telegram",
                value: alice_contact_handler,
                account_id: 0
            }
        }, {});
        expect(get_account_for_contact_alice_missing_account_id).not.toBe(alice);

        const get_account_for_contact_alice_missing_category = await auth.view("get_account_for_contact", {
            contact: {
                category: "Twitter",
                value: alice_contact_handler,
                account_id: Number(bob_contact_id)
            }
        }, {});
        expect(get_account_for_contact_alice_missing_category).not.toBe(alice);

    });

    test("Whitelist key, remove key, check deposit", async () => {
        const storage_withdraw = await auth.call("storage_withdraw", {}, {account_id: alice});
        expect(storage_withdraw.type).not.toBe('FunctionCallError');

        const already_has_request = await auth.view("has_request_key", {account_id: alice}, {});
        if (already_has_request)
            await auth.call("remove_request", {}, {account_id: alice});

        const storage_deposit = await auth.call("storage_deposit", {}, {
            account_id: alice,
            tokens: utils.ConvertToNear(0.1)
        });
        expect(storage_deposit.type).not.toBe('FunctionCallError');

        const storage_paid_1 = await auth.viewNearBalance("storage_paid", {account_id: alice}, {});

        const whitelist_key = await auth.call("whitelist_key", {
            account_id: alice,
            request_key: request_key,
        }, {
            account_id: admin
        });
        expect(whitelist_key.type).not.toBe('FunctionCallError');

        const remove = await auth.call("remove_request", {}, {account_id: alice});
        expect(remove.type).not.toBe('FunctionCallError');

        const storage_paid_2 = await auth.viewNearBalance("storage_paid", {account_id: alice}, {});

        expect(utils.RoundFloat(storage_paid_1 - storage_paid_2)).toBeCloseTo(WHITELIST_FEE, 5);
    });

    test('Bob deposit & send tips to Alice with auth', async () => {
        await near.call("withdraw_from_telegram", {telegram_account: alice_contact_id, account_id: alice},
            {account_id: admin});

        const deposit = await near.call("deposit", {}, {
            account_id: bob,
            tokens: utils.ConvertToNear(deposit_size)
        });
        expect(deposit.type).not.toBe('FunctionCallError');

        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});
        const alice_wallet_balance_1 = await near.accountNearBalance(alice);
        const alice_tips_1 = await near.viewNearBalance("get_balance", {telegram_account: alice_contact_id});

        const send_tip_to_telegram_with_auth = await near.call("send_tip_to_telegram_with_auth", {
            telegram_account: alice_contact_id, amount: utils.ConvertToNear(tip_size)
        }, {account_id: bob, gas: 200000000000000});
        expect(send_tip_to_telegram_with_auth.type).not.toBe('FunctionCallError');

        const alice_tips = await near.viewNearBalance("get_balance", {telegram_account: alice_contact_id});
        expect(utils.RoundFloat(alice_tips)).toBeCloseTo(0, 1);

        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});
        const alice_wallet_balance_2 = await near.accountNearBalance(alice);
        const alice_tips_2 = await near.viewNearBalance("get_balance", {telegram_account: alice_contact_id});

        expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(0, 1);
        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBeCloseTo(tip_size, 1);
        expect(utils.RoundFloat(alice_tips_2 - alice_tips_1)).toBe(0);

        // send to acc without auth

        const carol_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: carol});
        const carol_wallet_balance_1 = await near.accountNearBalance(carol);
        const carol_tips_1 = await near.viewNearBalance("get_balance", {telegram_account: carol_contact_id});

        const send_tip_to_telegram_with_auth_no_auth = await near.call("send_tip_to_telegram_with_auth", {
            telegram_account: carol_contact_id, amount: utils.ConvertToNear(tip_size)
        }, {account_id: bob, gas: 200000000000000});
        expect(send_tip_to_telegram_with_auth_no_auth.type).not.toBe('FunctionCallError');

        const carol_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: carol});
        const carol_wallet_balance_2 = await near.accountNearBalance(carol);
        const carol_tips_2 = await near.viewNearBalance("get_balance", {telegram_account: carol_contact_id});

        expect(utils.RoundFloat(carol_deposit_1)).toBe(carol_deposit_2);
        expect(utils.RoundFloat(carol_wallet_balance_2 - carol_wallet_balance_1)).toBeCloseTo(0, 1);
        expect(utils.RoundFloat(carol_tips_2 - carol_tips_1)).toBe(tip_size);
    });

    test('Bob deposit & send tips to Alice, withdraw with auth', async () => {
        await near.call("withdraw_from_telegram", {telegram_account: alice_contact_id, account_id: alice},
            {account_id: admin});

        const deposit = await near.call("deposit", {}, {
            account_id: bob,
            tokens: utils.ConvertToNear(deposit_size)
        });
        expect(deposit.type).not.toBe('FunctionCallError');

        const send_tip_to_telegram = await near.call("send_tip_to_telegram", {
            telegram_account: alice_contact_id,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: bob
        });
        expect(send_tip_to_telegram.type).not.toBe('FunctionCallError');

        const alice_tips = await near.viewNearBalance("get_balance", {telegram_account: alice_contact_id});
        expect(utils.RoundFloat(alice_tips)).toBeCloseTo(tip_size, 1);

        const alice_wallet_balance_1 = await near.accountNearBalance(alice);

        const withdraw_from_telegram_with_auth_alice = await near.call("withdraw_from_telegram_with_auth",
            {"telegram_account": Number(alice_contact_id)}, {account_id: alice, gas: 200000000000000});
        expect(withdraw_from_telegram_with_auth_alice.type).not.toBe('FunctionCallError');

        const alice_wallet_balance_2 = await near.accountNearBalance(alice);
        expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(alice_tips, 1);

        const withdraw_from_telegram_with_auth_bob = await near.call("withdraw_from_telegram_with_auth",
            {"telegram_account": Number(bob_contact_id)}, {account_id: alice, gas: 200000000000000});
        expect(withdraw_from_telegram_with_auth_bob.type).toBe('FunctionCallError');

        const withdraw_from_telegram_with_auth_bob_with_alice_contact = await near.call("withdraw_from_telegram_with_auth",
            {"telegram_account": Number(alice_contact_id)}, {account_id: bob, gas: 200000000000000});
        expect(withdraw_from_telegram_with_auth_bob_with_alice_contact.type).toBe('FunctionCallError');

        const withdraw_from_telegram_with_auth_carol = await near.call("withdraw_from_telegram_with_auth",
            {"telegram_account": Number(carol_contact_id)}, {account_id: carol, gas: 200000000000000});
        expect(withdraw_from_telegram_with_auth_carol.type).toBe('FunctionCallError');

        const withdraw_from_telegram_with_auth_alice_with_telegram_handler = await near.call("withdraw_from_telegram_with_auth",
            {"telegram_account": alice_contact_handler}, {account_id: alice, gas: 200000000000000});
        expect(withdraw_from_telegram_with_auth_alice_with_telegram_handler.type).toBe('FunctionCallError');

        const withdraw_from_telegram_with_auth_alice_again = await near.call("withdraw_from_telegram_with_auth",
            {"telegram_account": Number(alice_contact_id)}, {account_id: alice, gas: 200000000000000});
        expect(withdraw_from_telegram_with_auth_alice_again.type).toBe('FunctionCallError');

        const alice_wallet_balance_3 = await near.accountNearBalance(alice);
        expect(utils.RoundFloat(alice_wallet_balance_3)).toBeCloseTo(alice_wallet_balance_2, 0);
    });

    test('Bob deposit & send FT to Alice, withdraw with auth', async () => {
        await near.call("withdraw_from_telegram",
            {telegram_account: alice_contact_id, account_id: alice, token_id: ft_contract_account_id},
            {account_id: admin});

        const alice_deposit_1 = await ft.call("ft_transfer_call",
            {receiver_id: tipbot_account_id, amount: ft_deposit_size, msg: ""},
            {account_id: bob, tokens: 1});
        expect(alice_deposit_1.type).not.toBe('FunctionCallError');

        const send_ft_to_telegram = await near.call("send_tip_to_telegram", {
            telegram_account: alice_contact_id,
            amount: ft_tip_size,
            token_id: ft_contract_account_id
        }, {
            account_id: bob
        });
        expect(send_ft_to_telegram.type).not.toBe('FunctionCallError');

        const alice_ft_tips_1 = await near.viewDaiBalance("get_balance",
            {telegram_account: alice_contact_id, token_id: ft_contract_account_id});
        expect(utils.RoundFloat(alice_ft_tips_1)).toBe(utils.ConvertFromDai(ft_tip_size));

        const alice_ft_balance_1 = await ft.viewDaiBalance("ft_balance_of", {account_id: alice});

        const withdraw_from_telegram_with_auth_alice = await near.call("withdraw_from_telegram_with_auth",
            {"telegram_account": alice_contact_id, token_id: ft_contract_account_id},
            {account_id: alice, gas: 200000000000000});
        expect(withdraw_from_telegram_with_auth_alice.type).not.toBe('FunctionCallError');

        const alice_ft_balance_2 = await ft.viewDaiBalance("ft_balance_of", {account_id: alice});
        expect(utils.RoundFloat(alice_ft_balance_2 - alice_ft_balance_1)).toBe(utils.ConvertFromDai(ft_tip_size));

        const alice_ft_tips_2 = await near.viewDaiBalance("get_balance",
            {telegram_account: alice_contact_id, token_id: ft_contract_account_id});
        expect(utils.RoundFloat(alice_ft_tips_2)).toBe(0);
    });


    test("Tip contact direct to contact deposit: bob => alice", async () => {
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        await near.call("deposit", {}, {account_id: bob, tokens: utils.ConvertToNear(deposit_size)});

        const bob_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: bob});
        const alice_balance_1 = await near.viewNearBalance("get_balance", {telegram_account: alice_contact_id});
        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});

        const tx = await near.call("tip_contact_to_deposit", {
            telegram_account: Number(alice_contact_id),
            amount: utils.ConvertToNear(tip_size)
        }, {
            account_id: bob
        });
        expect(tx.type).not.toBe('FunctionCallError');

        const bob_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: bob});
        const alice_balance_2 = await near.viewNearBalance("get_balance", {telegram_account: alice_contact_id});
        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});

        expect(utils.RoundFloat(bob_deposit_1 - bob_deposit_2)).toBeCloseTo(tip_size, 5);
        expect(utils.RoundFloat(alice_balance_2 - alice_balance_1)).toBe(0);
        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBeCloseTo(tip_size, 5);

        // lets repeat a tip
        const tx2 = await near.call("tip_contact_to_deposit", {
            telegram_account: Number(alice_contact_id),
            amount: utils.ConvertToNear(tip_size)
        }, {
            account_id: bob
        });
        expect(tx2.type).not.toBe('FunctionCallError');

        const bob_deposit_3 = await near.viewNearBalance("get_deposit", {account_id: bob});
        const alice_deposit_3 = await near.viewNearBalance("get_deposit", {account_id: alice});

        expect(utils.RoundFloat(bob_deposit_1 - bob_deposit_3)).toBeCloseTo(tip_size * 2, 5);
        expect(utils.RoundFloat(alice_deposit_3 - alice_deposit_1)).toBeCloseTo(tip_size * 2, 5);

    });

    test("Tip attached tokens to contact", async () => {
        const generic_tips_available = await near.view("get_generic_tips_available", {});
        if (generic_tips_available) {

            const withdraw_tip_init = await near.call("withdraw_tip", {
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: alice_contact_id,
                    hint: "Tip attached tokens to contact"
                }
            }, {account_id: alice, gas: 200000000000000});
            expect(withdraw_tip_init.type).not.toBe('FunctionCallError');

            const alice_wallet_balance_1 = await near.accountNearBalance(alice);
            const alice_tip_by_contact_1 = await near.viewNearBalance("get_tip_by_contact", {
                account_id: alice,
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: alice_contact_id
                }
            });

            const tx = await near.call("tip_contact_with_attached_tokens", {
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: alice_contact_id
                }
            }, {
                account_id: bob,
                tokens: utils.ConvertToNear(tip_size)
            });
            expect(tx.type).not.toBe('FunctionCallError');

            const alice_tip_by_contact_2 = await near.viewNearBalance("get_tip_by_contact", {
                account_id: alice,
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: alice_contact_id
                }
            });
            expect(utils.RoundFloat(alice_tip_by_contact_2 - alice_tip_by_contact_1)).toBeCloseTo(tip_size, 5);

            const withdraw_tip = await near.call("withdraw_tip", {
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: Number(alice_contact_id)
                }
            }, {account_id: alice, gas: 200000000000000});
            expect(withdraw_tip.type).not.toBe('FunctionCallError');

            const alice_wallet_balance_2 = await near.accountNearBalance(alice);
            expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(tip_size, 0);
        }
    });

    test("Tip attached tokens to account by it contact", async () => {
        const generic_tips_available = await near.view("get_generic_tips_available", {});
        if (generic_tips_available) {
            const withdraw_tip_init = await near.call("withdraw_tip", {
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: Number(alice_contact_id)
                }
            }, {account_id: alice, gas: 200000000000000});
            expect(withdraw_tip_init.type).not.toBe('FunctionCallError');

            const alice_wallet_balance_1 = await near.accountNearBalance(alice);
            const alice_tip_by_contact_1 = await near.viewNearBalance("get_tip_by_contact", {
                account_id: alice,
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: Number(alice_contact_id)
                }
            });

            const tx = await near.call("tip_with_attached_tokens", {
                receiver_account_id: alice,
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: Number(alice_contact_id)
                }
            }, {
                account_id: bob,
                tokens: utils.ConvertToNear(tip_size)
            });
            expect(tx.type).not.toBe('FunctionCallError');

            const alice_tip_by_contact_2 = await near.viewNearBalance("get_tip_by_contact", {
                account_id: alice,
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: Number(alice_contact_id)
                }
            });
            expect(utils.RoundFloat(alice_tip_by_contact_2 - alice_tip_by_contact_1)).toBeCloseTo(tip_size, 5);

            const withdraw_tip = await near.call("withdraw_tip", {
                contact: {
                    category: "Telegram",
                    value: alice_contact_handler,
                    account_id: Number(alice_contact_id)
                }
            }, {account_id: alice, gas: 200000000000000});
            expect(withdraw_tip.type).not.toBe('FunctionCallError');

            const alice_wallet_balance_2 = await near.accountNearBalance(alice);
            expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(tip_size, 0);
        }
    });

    test("Withdraw storage", async () => {
        const storage_withdraw = await auth.call("storage_withdraw", {}, {account_id: alice});
        expect(storage_withdraw.type).not.toBe('FunctionCallError');
    });

    test("Remove all contacts", async () => {
        const is_owner = await auth.view("is_owner", {
            account_id: alice,
            contact: {category: "Telegram", value: alice_contact_handler, account_id: Number(alice_contact_id)}
        }, {});
        expect(is_owner).toBeTruthy();

        const get_account_for_contact_1 = await auth.view("get_account_for_contact", {
            contact: {category: "Telegram", value: alice_contact_handler, account_id: Number(alice_contact_id)}
        }, {});
        expect(get_account_for_contact_1).toBe(alice)

        const remove = await auth.call("remove_all", {}, {account_id: alice});
        expect(remove.type).not.toBe('FunctionCallError');

        const is_owner_2 = await auth.view("is_owner", {
            account_id: alice,
            contact: {category: "Telegram", value: alice_contact_handler, account_id: Number(alice_contact_id)}
        }, {});
        expect(is_owner_2).not.toBeTruthy();

        const get_account_for_contact_2 = await auth.view("get_account_for_contact", {
            contact: {category: "Telegram", value: alice_contact_handler, account_id: Number(alice_contact_id)}
        }, {});
        expect(get_account_for_contact_2).not.toBe(alice)
    })
});


function get_service_account(telegram_id) {
    return {
        service: "Telegram",
        account_id: telegram_id,
        account_name: null,
    };
}