/* export REACT_CONTRACT_ID=dev-1627393733545-88687685295664 */
import 'regenerator-runtime/runtime'

const contract = require('./rest-api-test-utils');
const utils = require('./utils');

const alice = "grant.testnet";
const alice_contact_handler = "alice_contact";
const alice_contact_id = "123";
const bob = "place.testnet";
const bob_contact_handler = "bob_contact";
const bob_contact_id = "456";
const admin = "zavodil.testnet";
const carol = process.env.REACT_CONTRACT_ID; // without contacts
const carol_contact = "";
const carol_contact_id = "";


const deposit_size = 2.678;
const tip_size = 0.1234;
const admin_commission = 0.003;

const near = new contract(process.env.REACT_CONTRACT_ID);

const MAX_TOKENS_TO_TIP = 20;

describe("Contract set", () => {
    test("Contract is not null " + process.env.REACT_CONTRACT_ID, async () => {
        expect(process.env.REACT_CONTRACT_ID).not.toBe(undefined)
    });

    test('Accounts has enough funds', async () => {
        const alice_wallet_balance = await near.accountNearBalance(alice);
        expect(alice_wallet_balance).toBeGreaterThan(MAX_TOKENS_TO_TIP);
    });
});


describe("Insert dummy data", () => {
    test('Insert deposits', async () => {

        for (let i = 1; i <= 200; i++) {
            let account = `account_${i}.testnet`;
            let tokens = i / 1000;
            const deposit_to_account = await near.call("deposit_to_account", {account_id: account},
                {account_id: admin, tokens: utils.ConvertToNear(tokens)});
            expect(deposit_to_account.type).not.toBe('FunctionCallError');
        }
    });

    test('Insert tips', async () => {
        const deposit = await near.call("deposit", {},
            {account_id: alice, tokens: utils.ConvertToNear(MAX_TOKENS_TO_TIP)});

        expect(deposit.type).not.toBe('FunctionCallError');

        for (let i = 1; i <= 1000; i++) {
            let telegram_account = `1000${i}`;
            let tip = i / 1000;
            const deposit_to_account = await near.call("send_tip_to_telegram",
                {telegram_account: telegram_account, amount: utils.ConvertToNear(tip)},
                {account_id: alice});
            expect(deposit_to_account.type).not.toBe('FunctionCallError');
        }
    });
});