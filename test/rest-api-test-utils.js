const fetch = require("node-fetch");
const utils = require('./utils');


const API_SERVER_URL = "https://rest.nearapi.org";
const CONTRACT_ACCOUNT_ID = process.env.REACT_CONTRACT_ID;

const GAS = 100000000000000;

module.exports = {
    deploy: async (contractName) => {
        const body = {
            contract: contractName,
            account_id: CONTRACT_ACCOUNT_ID,
            private_key: await utils.getPrivateKey(CONTRACT_ACCOUNT_ID),
        };

        return await PostResponse("deploy", body);
    },

    view: async (method, params, options) => {
        const body = {
            method: method,
            params: params,
            contract: CONTRACT_ACCOUNT_ID,
            disabled_cache: true
        };

        return await PostResponse("view", body, options);
    },

    viewNearBalance: async (method, params, options) => {
        options = options || {};
        options.convertToNear = true;
        return await module.exports.view(method, params, options);
    },

    accountNearBalance: async (account_id) => {
        return await GetResponse("balance", account_id, {convertToNear: true});
    },

    call: async (method, params, options) => {
        options.attached_gas = options.gas || GAS;
        options.attached_tokens = options.tokens || 0;
        options.private_key = options.private_key || await utils.getPrivateKey(options.account_id);

        const body = {
            ...options,
            method: method,
            params: params,
            contract: CONTRACT_ACCOUNT_ID,
        };

        try {
            return await PostResponse("call", body);
        } catch (e) {
            throw new Error("Call error for " + JSON.stringify(body) + ". Error: " + e.message);
        }
    },
}

const PostResponse = async (operation, body, options) => {
    return fetch(`${API_SERVER_URL}/${operation}`, {
        method: 'POST',
        body: JSON.stringify(body),
        headers: {
            'Content-type': 'application/json; charset=UTF-8'
        }
    })
        .then(res => {
            if (options && options.convertToNear) {
                return res.text().then(value => {
                    try {
                        return utils.RoundFloat(utils.ConvertYoctoNear(value, utils.FRACTION_DIGITS));
                    } catch (e) {
                        throw new Error("PostResponse error for " + operation + " request " + JSON.stringify(body) + ". Error: " + e.message);
                    }
                });
            } else {
                return res.json().then(json => {
                    try {
                        if (json.error)
                            return (JSON.parse(json.error));
                        else
                            return (json);
                    } catch (e) {
                        throw new Error("PostResponse error for " + operation + " request " + JSON.stringify(body) + ". Error: " + e.message);
                    }
                });
            }
        });
};

const GetResponse = async (operation, value, options) => {
    return fetch(`${API_SERVER_URL}/${operation}/${value}`, {
        method: 'GET'
    })
        .then(res => {
            if (options && options.convertToNear) {
                return res.text().then(value => {
                    try {
                        return utils.RoundFloat(utils.ConvertYoctoNear(value, utils.FRACTION_DIGITS));
                    } catch (e) {
                        throw new Error("GetResponse error for " + operation + " request " + JSON.stringify(body) + ". Error: " + e.message);
                    }
                });
            } else {
                return res.json().then(json => {
                    try {
                        if (json.error)
                            return (JSON.parse(json.error));
                        else
                            return (json);
                    } catch (e) {
                        throw new Error("GetResponse error for " + operation + " request " + JSON.stringify(body) + ". Error: " + e.message);
                    }
                });
            }
        });
};