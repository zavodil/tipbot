const utils = require('./utils');
const config = require("./config");

function contract(contract_id) {
    this.contract_id = contract_id;
}

contract.prototype.deploy = async function (contractName) {
    const body = {
        contract: contractName,
        account_id: this.contract_id,
        private_key: await utils.getPrivateKey(this.contract_id),
    };

    return await utils.PostResponse("deploy", body);
};

contract.prototype.view = async function (method, params, options) {
    params = params || {};
    options = options || {};

    const body = {
        method: method,
        params: params,
        contract: this.contract_id,
        disabled_cache: true
    };

    if (options !== undefined && options.rpc_node) {
        body.rpc_node = options.rpc_node;
    }

    return await utils.PostResponse("view", body, options);
};

contract.prototype.viewNearBalance = async function (method, params, options) {
    options = options || {};
    options.convertToNear = true;
    return await this.view(method, params, options);
};

contract.prototype.viewDaiBalance = async function (method, params, options) {
    options = options || {};
    options.convertFromDai = true;
    return await this.view(method, params, options);
};

contract.prototype.accountNearBalance = async function (account_id, delay) {
    delay = delay || 100;
    await timeout(delay);

    return await utils.GetResponse("balance", account_id, {convertToNear: true})
        .catch(e => console.error("AccountNearBalance error for " + JSON.stringify(account_id) + ". Error: " + e.message));
};

contract.prototype.call = async function (method, params, options) {
    params = params || {};
    if (!options.hasOwnProperty("account_id")) {
        throw new Error ("Account_id was not provided for CALL request")
    }

    options.attached_gas = options.attached_gas || options.gas || config.GAS;
    if(options.hasOwnProperty("deposit_near")){
        options.attached_tokens = utils.ConvertToNear(options.deposit_near);
    } else {
        options.attached_tokens = options.attached_tokens || options.deposit || 0;
    }
    options.private_key = options.private_key || await utils.getPrivateKey(options.account_id);
    options.log_errors = typeof (options.log_errors) ? options.log_errors : true;
    options.return_value = options.return_value || false;
    options.return_value_int = options.return_value_int || false;
    options.return_value_float = options.return_value_float || false;
    options.return_json = options.return_json || false;


    const body = {
        ...options,
        method: method,
        params: params,
        contract: this.contract_id,
    };

    return await utils.PostResponse("call", body, options)
        .catch(e => {
            if (e.message.includes("Unexpected token < in JSON at position 0"))
                console.error("RPC/JSON Error: " + e.message);
            else
                console.error("Call error for " + JSON.stringify(body) + ". Error: " + e.message)
        });
};

function timeout(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

module.exports = contract;
