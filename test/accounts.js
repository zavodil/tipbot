// SELECT * FROM user_keys WHERE contract_name = "tipbot.app.near" AND network = "Mainnet"

const nearAPI = require("near-api-js");
const fs = require("fs");

const CONTRACT_ID = "tipbot.app.near";

let portion = 5000;
let step = 0;

const log_file = fs.createWriteStream(__dirname + `/accounts_${step}.log`, {flags : 'w'});

async function connect () {
    const config = {
        networkId: "mainnet",
        keyStore: new nearAPI.keyStores.InMemoryKeyStore(),
        nodeUrl: "https://rpc.mainnet.near.org",
        walletUrl: "https://wallet.mainnet.near.org",
        helperUrl: "https://helper.mainnet.near.org",
        explorerUrl: "https://explorer.mainnet.near.org",
    };
    const near = await nearAPI.connect(config);
    const account = await near.account(CONTRACT_ID);

    const contract = new nearAPI.Contract(
        account, // the account object that is connecting
        CONTRACT_ID,
        {
            // name of contract you're connecting to
            viewMethods: ["get_deposit", "get_balance"], // view methods do not change state but usually return a value
            changeMethods: [], // change methods modify state
            sender: account, // account object to initialize and sign transactions.
        }
    );

    return contract;
}

connect().then(async contract => {



    let accounts = fs.readFileSync("./all_accounts.txt").toString('utf-8').split("\n");

    let accounts_to_transfer_deposit = [];
    let accounts_to_transfer_balance = [];
    let accounts_to_transfer_dai = [];

    let deposits = [];
    let balances = [];
    let dais = [];

    for (let i = step * portion; i < (step + 1) * portion; i++) {
        if(!accounts[i])
            break;

        let data = accounts[i].split(";")
        let account_id = data[0];
        let telegram_account = Number(data[1]);

        let deposit = await contract.get_deposit({account_id});
        if (deposit > 0) {
            accounts_to_transfer_deposit.push(account_id);
            deposits.push([account_id, nearAPI.utils.format.formatNearAmount(deposit)]);
        }

        let balance_near = await contract.get_balance({telegram_account});
        if (balance_near > 0) {
            accounts_to_transfer_balance.push(account_id);
            balances.push([account_id, nearAPI.utils.format.formatNearAmount(balance_near)]);
        }

        let balance_dai = await contract.get_balance({telegram_account, token_id: "6b175474e89094c44da98b954eedeac495271d0f.factory.bridge.near"});
        if (balance_dai > 0) {
            accounts_to_transfer_dai.push(account_id);
            dais.push([account_id, balance_dai]);
        }
    }

    console.dir("dais");
    console.dir(dais, {'maxArrayLength': null});
    log_file.write("dais");
    log_file.write(JSON.stringify(dais) + '\n');
    console.dir("balances");
    console.dir(balances, {'maxArrayLength': null});
    log_file.write("balances");
    log_file.write(JSON.stringify(balances) + '\n');
    console.dir("deposits");
    console.dir(deposits, {'maxArrayLength': null});
    log_file.write("deposits");
    log_file.write(JSON.stringify(deposits) + '\n');


    console.dir("accounts_to_transfer_dai");
    console.dir(accounts_to_transfer_dai, {'maxArrayLength': null});
    log_file.write("accounts_to_transfer_dai\n");
    log_file.write(JSON.stringify(accounts_to_transfer_dai) + '\n');
    console.dir("accounts_to_transfer_balance");
    console.dir(accounts_to_transfer_balance, {'maxArrayLength': null});
    log_file.write("accounts_to_transfer_balance\n");
    log_file.write(JSON.stringify(accounts_to_transfer_balance) + '\n');
    console.dir("accounts_to_transfer_deposit");
    console.dir(accounts_to_transfer_deposit, {'maxArrayLength': null});
    log_file.write("accounts_to_transfer_deposit\n");
    log_file.write(JSON.stringify(accounts_to_transfer_deposit) + '\n');

});


