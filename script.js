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

const STAKING_CONTRACT_ID = "uon_staking_test_09.xuguangxia.testnet";
const TOKEN_CONTRACT_ID = process.env.REACT_APP_TOKEN_CONTRACT_ID;
const REF_CONTRACT_ID = "ref-finance-101.testnet";
const WRAP_CONTRACT_ID = "wrap.testnet";

const Test = async () => {
  //Load Your Account
  const near = await connect(config);

  // STEP 4 enter your mainnet or testnet account name here!
  const contract_account = await near.account(
    "uon_staking_test_09.xuguangxia.testnet"
  );
  const xu_account = await near.account("xuguangxia.testnet");

  let result;

  // result = await xu_account.functionCall({
  //   contractId: STAKING_CONTRACT_ID,
  //   methodName: "save_swap_farm",
  //   args: {
  //     token_id: "token_test01.supernova11.testnet",
  //     swap_rate: "10000000",
  //     decimal: 24,
  //     min_lock_time: 86400000,
  //     max_lock_time: 2 * 86400000,
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Add Swap Farm");

  // result = await xu_account.functionCall({
  //   contractId: STAKING_CONTRACT_ID,
  //   methodName: "save_swap_farm",
  //   args: {
  //     token_id: "token_test02.supernova11.testnet",
  //     swap_rate: "30000000",
  //     decimal: 20,
  //     min_lock_time: 86400000,
  //     max_lock_time: 2 * 86400000,
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("Add Swap Farm");


  // result = await xu_account.functionCall({
  //   contractId: STAKING_CONTRACT_ID,
  //   methodName: "add_admin",
  //   args: {
  //     account_id: "pocktest.testnet",
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "0",
  // });
  // console.log("Add Admin");

  result = await xu_account.functionCall({
    contractId: STAKING_CONTRACT_ID,
    methodName: "remove_admin",
    args: {
      account_id: "dekatjauh.testnet",
    },
    gas: MAX_GAS,
    attachedDeposit: "0",
  });
  console.log("Remove Admin");

  // result = await contract_account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "storage_deposit",
  //   args: {
  //     account_id: STAKING_CONTRACT_ID,
  //     registration_only: false,
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1020000000000000000000",
  // });
  // console.log("register token");

  // result = await contract_account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "mft_register",
  //   args: {
  //     account_id: STAKING_CONTRACT_ID,
  //     token_id: ":382",
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1040000000000000000000",
  // });
  // console.log("register contract");

  // result = await contract_account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "mft_register",
  //   args: {
  //     account_id: STAKING_CONTRACT_ID,
  //     token_id: ":642",
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1040000000000000000000",
  // });
  // console.log("register contract");

  // const admin_list = ["halyconstudio.testnet", "decentricity.testnet", "francishor.testnet", "cj.testnet", "pocktest.testnet", "nearfar.testnet", "marvin786.testnet"];
  // for (let i = 0; i < admin_list.length; i++) {
  //   result = await xu_account.functionCall({
  //     contractId: STAKING_CONTRACT_ID,
  //     methodName: "remove_admin",
  //     args: {
  //       account_id: admin_list[i]
  //     },
  //     gas: MAX_GAS,
  //     attachedDeposit: "0",
  //   });
  //   console.log("Add Admin");
  // }

  // result = await xu_account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "add_simple_pool",
  //   args: {
  //     tokens: [
  //       "token_test02.supernova11.testnet",
  //       "usdn.testnet"
  //     ],
  //     fee: 30
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "100000000000000000000000",
  // });
  // console.log("create pool contract");

  // get lp balance of wallet
  // result = await provider.query({
  //   request_type: "call_function",
  //   account_id: REF_CONTRACT_ID,
  //   method_name: "get_pool_shares",
  //   args_base64: btoa(
  //     `{"pool_id": 382, "account_id": "uon_staking_test_09.xuguangxia.testnet"}`
  //   ),
  //   finality: "optimistic",
  // });
  // var tmp = JSON.parse(Buffer.from(result.result).toString());
  // console.log(tmp, ">>>> get balance of lp")

  // transfer unet from SC to supernova -----------------
  // let rawResult = await provider.query({
  //   request_type: "call_function",
  //   account_id: TOKEN_CONTRACT_ID,
  //   method_name: "ft_balance_of",
  //   args_base64: btoa(
  //     `{"account_id": "uon_staking_test_09.xuguangxia.testnet"}`
  //   ),
  //   finality: "optimistic",
  // });
  // var tmp = JSON.parse(Buffer.from(rawResult.result).toString());
  // console.log(tmp, "unet balance")
  // result = await account.functionCall({
  //   contractId: TOKEN_CONTRACT_ID,
  //   methodName: "ft_transfer",//mft_register
  //   args: {
  //     receiver_id: "xuguangxia.testnet",
  //     amount: "8559478000000000000000",
  //     msg: "get back unet",
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1",
  // });
  // console.log("defund from contract to account");

  // transfer lp from SC to supernova -----------------
  // get lp balance
  // let rawResult = await provider.query({
  //   request_type: "call_function",
  //   account_id: REF_CONTRACT_ID,
  //   method_name: "get_pool_shares",
  //   args_base64: btoa(
  //     `{"pool_id": 382, "account_id": "uon_staking_test_09.xuguangxia.testnet"}`
  //   ),
  //   finality: "optimistic",
  // });
  // var tmp = JSON.parse(Buffer.from(rawResult.result).toString());
  // console.log(tmp, ">>>> get balance of lp of the address")

  // result = await account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "mft_transfer",//mft_register
  //   args: {
  //     // receiver_id: "xuguangxia.testnet",
  //     // amount: "" + 700 * 100000000 + "0000000000",
  //     // msg: "get back unet",

  //     token_id: ":382",
  //     receiver_id: "xuguangxia.testnet",
  //     amount: "1000000000000000000000",

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
  // result = await account.functionCall({
  //   contractId: REF_CONTRACT_ID,
  //   methodName: "mft_register",//mft_register
  //   args: {
  //     account_id: STAKING_CONTRACT_ID,
  //     token_id: ":382",
  //     // registration_only: false,
  //   },
  //   gas: MAX_GAS,
  //   attachedDeposit: "1040000000000000000000",
  // });
  // console.log("register contract");
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
};

Test();
