const {utils} = require("near-api-js");
const path = require("path");
const homedir = require("os").homedir();
const {BN} = require('bn.js');
const fs = require('fs');
const fetch = require("node-fetch");
const config = require("./config");
const helper = require("./helper");


module.exports = {
    ConvertYoctoNear: (value, frac_digits) => {
        try {
            return utils.format.formatNearAmount(value, frac_digits).replace(",", "");
        } catch (e) {
            console.log("ConvertYoctoNear error, value: " + value);
            console.log(e);
        }
    },

    ConvertFromDai: (value) => {
        return (Math.round(value / 100000000000000) / 10000);
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
            console.error("Key not found for account " + keyPath + ". Error: " + e.message);
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
                    if (options && options.log_errors) {
                        const response_json = JSON.parse(response);
                        if (response_json && response_json.error) {
                            const error = JSON.parse(response_json.error);
                            console.log(`Request: ${body.method}`);
                            console.log(`ERROR: ${error.type}: ${JSON.stringify(error.kind)}`);
                        }
                    }
                    if (options && options.return_value) {
                        return response;
                    }
                    if (options && (options.convertToNear || options.convertFromDai)) {
                        if (isNaN(response))
                            throw new Error(`Illegal balance value. Request: ${JSON.stringify(body)}. Response: ${response}`);

                        if (options.convertFromDai)
                            return module.exports.RoundFloat(module.exports.ConvertFromDai(response, config.FRACTION_DIGITS));
                        else
                            return module.exports.RoundFloat(module.exports.ConvertYoctoNear(response, config.FRACTION_DIGITS));
                    } else {
                        try {
                            const json = JSON.parse(response);
                            if (json.hasOwnProperty('error')) {
                                const error = JSON.parse(json.error);
                                if(options.log_errors && error.hasOwnProperty('transaction_outcome')) {
                                    console.log("Call error:" + helper.GetTxUrl(error.transaction_outcome.id));
                                }
                                return error;
                            }

                            try {
                                if (options.return_value_int || options.return_value_float || options.return_json) {
                                    if (json.hasOwnProperty("status")) {
                                        let value = Buffer.from(json.status.SuccessValue, 'base64').toString();
                                        if (options.return_value_int) {
                                            return parseInt(value);
                                        } else if (options.return_value_float) {
                                            return parseFloat(value);
                                        } else if (options.return_json) {
                                            return JSON.parse(json);
                                        } else
                                            return value;
                                    } else {
                                        if (options.return_json) {
                                            return JSON.parse(json);
                                        }
                                        else {
                                            return json;
                                        }
                                    }
                                } else {
                                    return (json);
                                }
                            } catch (e) {
                                throw new Error("PostResponse error for " + operation + " request " + JSON.stringify(body) + ". Error: " + e.message);
                            }
                        } catch (e) {
                            throw new Error("PostResponse error for " + operation + " request " + JSON.stringify(body) + ". Error: " + e.message);
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
    },

    GetTimestamp: (minutes_until_timestamp) => {
        return Number(((Date.now() + minutes_until_timestamp * 60 * 1000).toString() + "000000"));
    }
};
