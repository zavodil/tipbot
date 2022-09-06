import 'regenerator-runtime/runtime'

const contract = require('./rest-api-test-utils');
const utils = require('./utils');
const helper = require('./helper');

const alice = "alice.testnet";
const bob = "bob.testnet";
const carol = "carol.testnet";
const admin = "admin.testnet";

const contract_id = process.env.CONTRACT_NAME;
const near = new contract(contract_id);

describe("Contract set", () => {
    test("Contract is not null " + helper.GetContractUrl(), async () => {
        expect(contract_id).not.toBe(undefined)
    });

    test("Init contract", async () => {
        await near.call("new", {
            owner_id: admin
        }, {account_id: contract_id, log_errors: false});
    });

    test('Accounts has enough funds', async () => {
        const alice_wallet_balance = await near.accountNearBalance(alice);
        expect(alice_wallet_balance).toBeGreaterThan(20);

        const bob_wallet_balance = await near.accountNearBalance(bob);
        expect(bob_wallet_balance).toBeGreaterThan(20);
    });
});

describe("Tests", () => {
    test("Test", async () => {

    });
});
