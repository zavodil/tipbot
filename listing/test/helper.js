import {getConfig} from './config';

let nearAPI = getConfig(process.env.NODE_ENV || "testnet");


module.exports = {
    GetContractUrl: () => {
        return (nearAPI.explorerUrl + "/accounts/" + process.env.CONTRACT_NAME);
    },

    GetTxUrl: (id) => {
        return (nearAPI.explorerUrl + "/transactions/" + id);
    },

    GetNanosec: (date) => {
        return date.toString() + "000000";
    },

    ConvertToPow18: (amount) => {
        return (Math.round(amount * 100000000)).toString() + "0000000000";
    }
}
