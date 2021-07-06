const {utils} = require("near-api-js");
const path = require("path");
const homedir = require("os").homedir();
const {BN} = require('bn.js');
const fs = require('fs');
const fetch = require("node-fetch");
const config = require("./config");


module.exports = {
    ConvertYoctoNear: (value, frac_digits) => {
        return utils.format.formatNearAmount(value, frac_digits).replace(",", "");
    },

    ConvertToNear: (amount) => {
        return new BN(Math.round(amount * 100000000)).mul(new BN("10000000000000000")).toString();
    },

    RoundFloat: (amount) => {
        return +Number.parseFloat(amount).toFixed(config.FRACTION_DIGITS);
    },

    getPrivateKey: async (accountId) => {
        const credentialsPath = path.join(homedir, config.CREDENTIALS_DIR);
        const keyPath = credentialsPath + accountId + '.json';
        try {
            const credentials = JSON.parse(fs.readFileSync(keyPath));
            return (credentials.private_key);
        } catch (e) {
            throw new Error("Key not found for account " + keyPath + ". Error: " + e.message);
        }
    },

    PostResponse: async (operation, body, options) => {
        const response = fetch(`${config.API_SERVER_URL}/${operation}`, {
            method: 'POST',
            body: JSON.stringify(body),
            headers: {
                'Content-type': 'application/json; charset=UTF-8'
            }
        })
            .then(res => {
                return res.text().then(response => {
                    if (options && options.convertToNear) {
                        if (isNaN(response))
                            throw new Error(`Illegal balance value. Request: ${JSON.stringify(body)}. Response: ${response}`);

                        return module.exports.RoundFloat(module.exports.ConvertYoctoNear(response, config.FRACTION_DIGITS));
                    } else {
                        try {
                            const json = JSON.parse(response);
                            try {
                                if (json.error)
                                    return (JSON.parse(json.error));
                                else {
                                    return (json);
                                }
                            } catch (e) {
                                throw new Error("PostResponse error for " + operation + " request " + JSON.stringify(body) + ". Error: " + e.message);
                            }
                        } catch {
                            return response;
                        }
                    }
                });

            });
        return response;
    },

    GetResponse: async (operation, value, options) => {
        const response = await fetch(`${config.API_SERVER_URL}/${operation}/${value}`, {
            method: 'GET'
        })
            .then(res => {
                if (options && options.convertToNear) {
                    return res.text().then(value => {
                        try {
                            return module.exports.RoundFloat(module.exports.ConvertYoctoNear(value, config.FRACTION_DIGITS));
                        } catch (e) {
                            throw new Error("GetResponse error for " + operation + " request " + JSON.stringify(value) + ". Error: " + e.message);
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
                            throw new Error("GetResponse error for " + operation + " request " + JSON.stringify(value) + ". Error: " + e.message);
                        }
                    });
                }
            });

        return response;
    },

    IsJson: (item) => {
        item = typeof item !== "string"
            ? JSON.stringify(item)
            : item;

        try {
            item = JSON.parse(item);
        } catch (e) {
            return false;
        }

        if (typeof item === "object" && item !== null) {
            return true;
        }

        return false;
    }
};