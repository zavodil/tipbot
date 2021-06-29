import 'regenerator-runtime/runtime'

const contract = require('./rest-api-test-utils');
const utils = require('./utils');

const alice = "grant.testnet";
const alice_contact = "alice.contact";
const bob = "place.testnet";
const bob_contact = "kakoilogin";
const admin = "zavodil.testnet";
const carol = process.env.REACT_CONTRACT_ID; // without contacts
const carol_contact = "";


const deposit_size = 12.345;
const tip_size = 1;
const admin_commission = 0.003;

const near = new contract(process.env.REACT_CONTRACT_ID);

describe("Contract set", () => {
    test("Contract is not null " + process.env.REACT_CONTRACT_ID, async () => {
        //const contractName = await near.deploy("tipbot.wasm");
        expect(process.env.REACT_CONTRACT_ID).not.toBe(undefined)
    });
});


describe("Permissions", () => {
    test('Tip available', async () => {
        const tip_available_init = await near.call("set_tip_available", {tip_available: true}, {account_id: admin});
        const withdraw_available_init = await near.call("set_withdraw_available", {withdraw_available: true}, {account_id: admin});

        await near.call("withdraw", {}, {account_id: alice});

        const deposit = await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        expect(deposit.type).not.toBe('FunctionCallError');

        const tip_unavailable = await near.call("set_tip_available", {tip_available: false}, {account_id: admin});
        expect(tip_unavailable.type).not.toBe('FunctionCallError');

        const send_tip_1 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact,
            amount: utils.ConvertToNear(tip_size),
        }, {
            account_id: alice
        });
        expect(send_tip_1.type).toBe('FunctionCallError');

        const tip_available = await near.call("set_tip_available", {tip_available: true}, {account_id: admin});
        expect(tip_available.type).not.toBe('FunctionCallError');

        const send_tip_2 = await near.call("send_tip_to_telegram", {
            telegram_account: bob_contact,
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

        const withdraw_available_illegal = await near.call("set_withdraw_available", {tip_available: false}, {account_id: alice});
        expect(withdraw_available_illegal.type).toBe('FunctionCallError');
    });
});


describe("Deposit and Withdraw", () => {
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
        const illegal_transfer = await near.call("transfer_tips_to_deposit", {
            telegram_account: bob_contact,
            account_id: alice,
        }, {
            account_id: alice
        });

        expect(illegal_transfer.type).toBe('FunctionCallError');
        expect(illegal_transfer.kind.ExecutionError).toMatch(/(No access)/i);
    });
});


const auth = new contract("dev-1620499613958-3096267");
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
                value: alice_contact
            }
        }, {});

        if (is_need_to_remove_contact) {
            const remove = await auth.call("remove", {
                contact: {
                    category: "Telegram",
                    value: alice_contact
                }
            }, {account_id: alice});
            expect(remove.type).not.toBe('FunctionCallError');
        }

        const start_auth = await auth.call("start_auth", {
            request_key: request_key,
            contact: {
                category: "Telegram",
                value: alice_contact
            }
        }, {
            account_id: alice,
            tokens: 1
        });

        expect(start_auth.type).not.toBe('FunctionCallError');

        const confirm_auth = await auth.call("confirm_auth", {
            key: request_secret,
        }, {account_id: alice});

        expect(confirm_auth.type).not.toBe('FunctionCallError');

        const is_owner = await auth.view("is_owner", {
            account_id: alice,
            contact: {
                category: "Telegram",
                value: alice_contact
            }
        }, {});

        expect(is_owner).toBeTruthy()
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


    describe("Deposit and Withdraw", () => {
        test('Bob deposit & send tips to Alice', async () => {
            const deposit = await near.call("deposit", {}, {
                account_id: bob,
                tokens: utils.ConvertToNear(deposit_size)
            });
            expect(deposit.type).not.toBe('FunctionCallError');

            const send_tip_to_telegram = await near.call("send_tip_to_telegram", {
                telegram_account: alice_contact,
                amount: utils.ConvertToNear(tip_size),
            }, {
                account_id: bob
            });
            expect(send_tip_to_telegram.type).not.toBe('FunctionCallError');

            const alice_tips = await near.viewNearBalance("get_balance", {telegram_account: alice_contact});
            const alice_wallet_balance_1 = await near.accountNearBalance(alice);

            const withdraw_from_telegram_with_auth_alice = await near.call("withdraw_from_telegram_with_auth", {}, {account_id: alice});
            expect(withdraw_from_telegram_with_auth_alice.type).not.toBe('FunctionCallError');

            const withdraw_from_telegram_with_auth_carol = await near.call("withdraw_from_telegram_with_auth", {}, {account_id: carol});
            expect(withdraw_from_telegram_with_auth_carol.type).toBe(undefined); // null because doesn't have contracts

            const alice_wallet_balance_2 = await near.accountNearBalance(alice);

            expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(alice_tips, 5);
        });
    });

    test("Tip contact direct to contact deposit: bob => alice", async () => {
        await near.call("deposit", {}, {account_id: alice, tokens: utils.ConvertToNear(deposit_size)});
        await near.call("deposit", {}, {account_id: bob, tokens: utils.ConvertToNear(deposit_size)});

        const bob_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: bob});
        const alice_balance_1 = await near.viewNearBalance("get_balance", {telegram_account: alice_contact});
        const alice_deposit_1 = await near.viewNearBalance("get_deposit", {account_id: alice});

        const tx = await near.call("tip_contact_to_deposit", {
            telegram_handler: alice_contact,
            amount: utils.ConvertToNear(tip_size)
        }, {
            account_id: bob
        });
        expect(tx.type).not.toBe('FunctionCallError');

        const bob_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: bob});
        const alice_balance_2 = await near.viewNearBalance("get_balance", {telegram_account: alice_contact});
        const alice_deposit_2 = await near.viewNearBalance("get_deposit", {account_id: alice});

        expect(utils.RoundFloat(bob_deposit_1 - bob_deposit_2)).toBeCloseTo(tip_size, 5);
        expect(utils.RoundFloat(alice_balance_2 - alice_balance_1)).toBe(0);
        expect(utils.RoundFloat(alice_deposit_2 - alice_deposit_1)).toBeCloseTo(tip_size, 5);

        // lets repeat a tip
        const tx2 = await near.call("tip_contact_to_deposit", {
            telegram_handler: alice_contact,
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
        const alice_wallet_balance_1 = await near.accountNearBalance(alice);
        const alice_tip_by_contact_1 = await near.viewNearBalance("get_tip_by_contact", {
            account_id: alice,
            contact: {category: "Telegram", value: alice_contact}
        });

        const tx = await near.call("tip_contact_with_attached_tokens", {
            contact: {
                category: "Telegram",
                value: alice_contact
            }
        }, {
            account_id: bob,
            tokens: utils.ConvertToNear(tip_size)
        });
        expect(tx.type).not.toBe('FunctionCallError');

        const alice_tip_by_contact_2 = await near.viewNearBalance("get_tip_by_contact", {
            account_id: alice,
            contact: {category: "Telegram", value: alice_contact}
        });
        expect(utils.RoundFloat(alice_tip_by_contact_2 - alice_tip_by_contact_1)).toBeCloseTo(tip_size, 5);

        const withdraw_tip = await near.call("withdraw_tip", {
            contact: {
                category: "Telegram",
                value: alice_contact
            }
        }, {
            account_id: alice,
        });
        expect(withdraw_tip.type).not.toBe('FunctionCallError');

        const alice_wallet_balance_2 = await near.accountNearBalance(alice);
        expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(tip_size, 1);

    });

    test("Tip attached tokens to account by it contact", async () => {
        const alice_wallet_balance_1 = await near.accountNearBalance(alice);
        const alice_tip_by_contact_1 = await near.viewNearBalance("get_tip_by_contact", {
            account_id: alice,
            contact: {category: "Telegram", value: alice_contact}
        });

        const tx = await near.call("tip_with_attached_tokens", {
            receiver_account_id: alice,
            contact: {
                category: "Telegram",
                value: alice_contact
            }
        }, {
            account_id: bob,
            tokens: utils.ConvertToNear(tip_size)
        });
        expect(tx.type).not.toBe('FunctionCallError');

        const alice_tip_by_contact_2 = await near.viewNearBalance("get_tip_by_contact", {
            account_id: alice,
            contact: {category: "Telegram", value: alice_contact}
        });
        expect(utils.RoundFloat(alice_tip_by_contact_2 - alice_tip_by_contact_1)).toBeCloseTo(tip_size, 5);

        const withdraw_tip = await near.call("withdraw_tip", {
            contact: {
                category: "Telegram",
                value: alice_contact
            }
        }, {
            account_id: alice,
        });
        expect(withdraw_tip.type).not.toBe('FunctionCallError');

        const alice_wallet_balance_2 = await near.accountNearBalance(alice);
        expect(utils.RoundFloat(alice_wallet_balance_2 - alice_wallet_balance_1)).toBeCloseTo(tip_size, 1);

    });

    test("Withdraw storage", async () => {
        const storage_withdraw = await auth.call("storage_withdraw", {}, {account_id: alice});
        expect(storage_withdraw.type).not.toBe('FunctionCallError');
    });
});


