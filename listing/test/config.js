const CONTRACT_NAME = process.env.LISTING_CONTRACT_ID || 'near-contract'

module.exports = {
    API_SERVER_URL: "https://rest.nearapi.org",
    CREDENTIALS_DIR: ".near-credentials/testnet/",
    FRACTION_DIGITS: 5,
    GAS: 100000000000000,

    getConfig: function (env) {
        switch (env) {

            case 'production':
            case 'mainnet':
                return {
                    networkId: 'mainnet',
                    nodeUrl: 'https://rpc.mainnet.near.org',
                    contractName: CONTRACT_NAME,
                    walletUrl: 'https://wallet.near.org',
                    helperUrl: 'https://helper.mainnet.near.org',
                    explorerUrl: 'https://explorer.mainnet.near.org',
                }
            case 'development':
            case 'testnet':
                return {
                    networkId: 'testnet',
                    nodeUrl: 'https://rpc.testnet.near.org',
                    contractName: CONTRACT_NAME,
                    walletUrl: 'https://wallet.testnet.near.org',
                    helperUrl: 'https://helper.testnet.near.org',
                    explorerUrl: 'https://explorer.testnet.near.org',
                }
        }
    }
}
