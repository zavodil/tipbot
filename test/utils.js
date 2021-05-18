const {utils} = require("near-api-js");
const path = require("path");
const homedir = require("os").homedir();
import {BN} from 'bn.js'

const fs = require('fs');

const CREDENTIALS_DIR = ".near-credentials/testnet/";

module.exports = {
    FRACTION_DIGITS: 5,

    IsObject: (obj) => {
        return obj !== undefined && obj !== null && typeof obj == 'object';
    },

    ConvertYoctoNear: (value, frac_digits) => {
        return utils.format.formatNearAmount(value, frac_digits).replace(",", "");
    },

    ConvertToNear: (amount) => {
        return new BN(Math.round(amount * 100000000)).mul(new BN("10000000000000000")).toString();
    },

    RoundFloat: (amount) => {
        return +Number.parseFloat(amount).toFixed(module.exports.FRACTION_DIGITS);
    },

    getPrivateKey: async (accountId) => {
        const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
        const keyPath = credentialsPath + accountId + '.json';
        try {
            const credentials = JSON.parse(fs.readFileSync(keyPath));
            return (credentials.private_key);
        } catch (e) {
            throw new Error("Key not found for account " + keyPath + ". Error: " + e.message);
        }
    }
}