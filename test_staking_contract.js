// Welcome to the Mass Key Deletion recipe.

// This tool allows you to
// 1. Delete all your functionCall Access Keys
// 2. Delete all but one specified Full Access Key
// 3. Delete all Full Access Keys and Lock an Account

/// STEP 1 Install near-api-js
// npm init (in directory where you stored this script)
// npm i near-api-js

const nearAPI = require("near-api-js"); // imports near api js
const { providers } = require("near-api-js");
const provider = new providers.JsonRpcProvider("https://rpc.testnet.near.org");

// Standard setup to connect to NEAR While using Node
const { KeyPair, keyStores, connect } = nearAPI;
const homedir = require("os").homedir();
const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);
let config;

// STEP 2 Choose your configuration.
// set this variable to either "testnet" or "mainnet"
// if you haven't used this before use testnet to experiment so you don't lose real tokens by deleting all your access keys
const configSetting = "testnet";

const GAS_FOR_NFT_APPROVE = "20000000000000";
const GAS_FOR_RESOLVE_TRANSFER = "10000000000000";
const MAX_GAS = "300000000000000";
const DEPOSIT = "450000000000000000000";
const UNET_UNIT = 1000000000000000000;
// setting configuration based on input
switch (configSetting) {
  case "mainnet":
    config = {
      networkId: "mainnet",
      keyStore, // optional if not signing transactions
      nodeUrl: "https://rpc.mainnet.near.org",
      walletUrl: "https://wallet.near.org",
      helperUrl: "https://helper.mainnet.near.org",
      explorerUrl: "https://explorer.mainnet.near.org",
    };
    console.log("configuration set to mainnet ");

    break;

  case "testnet":
    config = {
      networkId: "testnet",
      keyStore, // optional if not signing transactions
      nodeUrl: "https://rpc.testnet.near.org",
      walletUrl: "https://wallet.testnet.near.org",
      helperUrl: "https://helper.testnet.near.org",
      explorerUrl: "https://explorer.testnet.near.org",
    };
    console.log("configuration set to testnet ");
    break;
  default:
    console.log(`please choose a configuration `);
}

const STAKING_CONTRACT_ID = process.env.REACT_APP_STAKING_CONTRACT_ID;
const TOKEN_CONTRACT_ID = process.env.REACT_APP_TOKEN_CONTRACT_ID;
const REF_CONTRACT_ID = process.env.REACT_APP_REF_CONTRACT_ID;
const WRAP_CONTRACT_ID = "wrap.testnet";

const Test = async () => {
  //Load Your Account
  const near = await connect(config);

  // STEP 4 enter your mainnet or testnet account name here!
  const account = await near.account("uon_staking_test_01.supernova11.testnet");

  let result;

  // get unet balance of wallet
  // result = await provider.query({
  //   request_type: "call_function",
  //   account_id: TOKEN_CONTRACT_ID,
  //   method_name: "ft_balance_of",
  //   args_base64: btoa(
  //     `{"account_id": "uon_staking_test_01.supernova11.testnet"}`
  //   ),
  //   finality: "optimistic",
  // });
  // tmp = JSON.parse(Buffer.from(result.result).toString());
  // console.log(tmp, ">>>> get unet balance of contract")

  // get lp balance of wallet
  // result = await provider.query({
  //   request_type: "call_function",
  //   account_id: REF_CONTRACT_ID,
  //   method_name: "get_pool_shares",
  //   args_base64: btoa(
  //     `{"pool_id": 382, "account_id": "uon_staking_test_01.supernova11.testnet"}`
  //   ),
  //   finality: "optimistic",
  // });
  // var tmp = JSON.parse(Buffer.from(result.result).toString());
  // console.log(tmp, ">>>> get balance of lp")

  //
  // result = await account.getAccessKeys();
  // let tokenKeyExist = false;
  // for(let i=0; i<result.length; i++){
  //   if(result[i].access_key.permission != 'FullAccess' && result[i].access_key.permission.FunctionCall.receiver_id == STAKING_CONTRACT_ID){
  //     tokenKeyExist = true;
  //     break;
  //   }
  // }
  // if(tokenKeyExist == false){
  //   console.log("Adding AccessKey to Token");
  //   const keyPair = KeyPair.fromRandom("ed25519");
  //   const publicKey = keyPair.publicKey.toString();
  //   await keyStore.setKey(config.networkId, publicKey, keyPair);
  //   await account.addKey(publicKey, STAKING_CONTRACT_ID, [], '250000000000000000000000');
  // }

  // result = await account.getAccessKeys();
  // let tokenKeyExist = false;
  // for (let i = 0; i < result.length; i++) {
  //   if (result[i].access_key.permission != 'FullAccess' && result[i].access_key.permission.FunctionCall.receiver_id == WRAP_CONTRACT_ID) {
  //     tokenKeyExist = true;
  //     break;
  //   }
  // }
  // if (tokenKeyExist == false) {
  //   console.log("Adding AccessKey to Token");
  //   const keyPair = KeyPair.fromRandom("ed25519");
  //   const publicKey = keyPair.publicKey.toString();
  //   await keyStore.setKey(config.networkId, publicKey, keyPair);
  //   await account.addKey(publicKey, WRAP_CONTRACT_ID, [], '250000000000000000000000');
  // }


  // transfer unet from SC to supernova -----------------
  // result = await account.functionCall({
  //   contractId: TOKEN_CONTRACT_ID,
  //   methodName: "ft_transfer",//mft_register
  //   args: {
  //     receiver_id: "supernova11.testnet",
  //     amount: "8559478000000000000000",
  //     msg: "get back unet",
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("defund from contract to account");

  // transfer lp from SC to supernova -----------------
  // result = await account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "mft_transfer",//mft_register
  //   args: {
  //     // receiver_id: "supernova11.testnet",
  //     // amount: "" + 700 * 100000000 + "0000000000",
  //     // msg: "get back unet",

  //     token_id: ":382",
  //     receiver_id: "supernova11.testnet",
  //     amount: "161000000000000000000000",

  //     // account_id: STAKING_CONTRACT_ID,
  //     //token_id: ":382",
  //     // registration_only: false,
  //   },
  //   gas: MAX_GAS,
  //   // attachedDeposit: "1020000000000000000000",
  //   attachedDeposit: "1",
  // });
  // console.log("defund from contract to account");

  //--------------------------------------------------------
  //register token
  // result = await account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "storage_deposit",//mft_register
  //   args: {
  //     account_id: STAKING_CONTRACT_ID,
  //     // token_id: ":382",
  //     registration_only: false,
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1020000000000000000000",
  // });
  // console.log("register token");

  //register contract-------------------------------------
  result = await account.functionCall({
    contractId: REF_CONTRACT_ID,
    methodName: "mft_register",//mft_register
    args: {
      account_id: STAKING_CONTRACT_ID,
      token_id: ":382",
      // registration_only: false,
    },
    gas: MAX_GAS,
    attachedDeposit: "1040000000000000000000",
  });
  console.log("register contract");
  //-------------------------------------------------

  // get lp back to me
  // await account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "remove_liquidity",
  //   args: {
  //     min_amounts: ["0", "0"],
  //     pool_id: 382,
  //     shares: "140000000000000000000000"
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",

  // })
  // console.log('ssuccess')



  // STAKING
  // result = await account.functionCall({
  //   contractId: STAKING_CONTRACT_ID,
  //   methodName: "add_farm",
  //   args: {
  //     token_id: TOKEN_CONTRACT_ID,
  //     pool_id: 547,
  //     reward_token_id: TOKEN_CONTRACT_ID,
  //     token_reward_rate: "10000000000",
  //     pool_reward_rate: "15000000000",
  //     starting_at: Date.now(),
  //     ending_at: Date.now() + 1000 * 60 * 60 * 24 * 7,
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Add Farm");


  // STAKING
  // result = await account.functionCall({
  //   contractId: TOKEN_CONTRACT_ID,
  //   methodName: "ft_transfer_call",
  //   args: {
  //     receiver_id: STAKING_CONTRACT_ID,
  //     amount: "100000000000",
  //     msg: JSON.stringify({ farm_id: 0, lock_duration: 1000*60*60*24 })
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Staking UNET token");

  // // STAKING
  // result = await account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "mft_transfer_call",
  //   args: {
  //     token_id: ":547",
  //     receiver_id: STAKING_CONTRACT_ID,
  //     amount: "10000000",
  //     msg: JSON.stringify({ farm_id: 0, lock_duration: 1000*60*60*24 })
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Staking LP token");

  // result = await account.viewFunction(
  //   STAKING_CONTRACT_ID,
  //   "get_claim_amount",
  //   {
  //     farm_id: 0,
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log(result);

  // if(result.total == '0' && result.available == '0'){
  // const depositAmount = await account.viewFunction(
  //   WRAP_CONTRACT_ID,
  //   "storage_minimum_balance",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // result = await account.functionCall({
  //   contractId: WRAP_CONTRACT_ID,
  //   methodName: "storage_deposit",
  //   args: {
  //     receiver_id: account.accountId,
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: depositAmount,
  // });
  // console.log('registered');
  //   console.log(result);
  // }


  // result = await account.viewFunction(
  //   TOKEN_A_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_A_Balance:", result);

  // result = await account.viewFunction(
  //   TOKEN_B_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_B_Balance:", result);

  // result = await account.viewFunction(
  //   TOKEN_C_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_C_Balance:", result);

  // STAKING
  // result = await account.functionCall({
  //   contractId: TOKEN_A_CONTRACT_ID,
  //   methodName: "ft_transfer_call",
  //   args: {
  //     receiver_id: STAKING_CONTRACT_ID,
  //     amount: "1000000000000000000000000",
  //     msg: JSON.stringify({ staking_status: "Stake to Platform" })
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Staking A token");

  // result = await account.functionCall({
  //   contractId: TOKEN_B_CONTRACT_ID,
  //   methodName: "ft_transfer_call",
  //   args: {
  //     receiver_id: STAKING_CONTRACT_ID,
  //     amount: "1000000000000000000000000",
  //     msg: JSON.stringify({ staking_status: "Stake to Platform" })
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Staking B token");

  // result = await account.viewFunction(
  //   TOKEN_A_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_A_Balance:", result);

  // result = await account.viewFunction(
  //   TOKEN_B_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_B_Balance:", result);

  // result = await account.viewFunction(
  //   TOKEN_C_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_C_Balance:", result);

  // result = await account.viewFunction(
  //   STAKING_CONTRACT_ID,
  //   "get_claim_amount",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("ClaimAmount:", result);

  // CLAIMING
  // result = await account.functionCall({
  //   contractId: STAKING_CONTRACT_ID,
  //   methodName: "claim_reward",
  //   args: {
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Claimed");

  // UNSTAKING
  // result = await account.functionCall({
  //   contractId: STAKING_CONTRACT_ID,
  //   methodName: "unstake",
  //   args: {
  //     token_type: TOKEN_A_CONTRACT_ID,
  //     amount: "500000000000000000000000"
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Unstaked A Token");

  // result = await account.viewFunction(
  //   TOKEN_A_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_A_Balance:", result);

  // result = await account.viewFunction(
  //   TOKEN_B_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_B_Balance:", result);

  // result = await account.viewFunction(
  //   TOKEN_C_CONTRACT_ID,
  //   "ft_balance_of",
  //   {
  //     account_id: account.accountId,
  //   }
  // ); 
  // console.log("Token_C_Balance:", result);
};

Test();
