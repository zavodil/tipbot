use crate::*;

#[near_bindgen]
impl NearTips {
    #[init(ignore_state)]
    #[allow(dead_code)]
    pub fn migrate_state_4() -> Self { // RewardPoint type updated , total_chat_points added
        let migration_version: u16 = 4;
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

        #[derive(BorshDeserialize)]
        struct OldContract {
            deposits: LookupMap<TokenByNearAccount, Balance>,
            telegram_tips: LookupMap<TokenByTelegramAccount, Balance>,
            tips: LookupMap<AccountId, Vec<Tip>>,
            telegram_users_in_chats: LookupSet<TelegramUserInChat>,
            chat_points: LookupMap<TokenByTelegramChat, RewardPoint>,
            whitelisted_tokens: LookupSet<TokenAccountId>,
            version: u16,
            withdraw_available: bool,
            tip_available: bool,
            generic_tips_available: bool,

            telegram_tips_v1: HashMap<String, Balance>,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self {
            deposits: old_contract.deposits,
            telegram_tips: old_contract.telegram_tips,
            tips: old_contract.tips,
            telegram_users_in_chats: old_contract.telegram_users_in_chats,
            chat_points_v1: LookupMap::new(StorageKey::ChatPointsLookupMapU128),
            whitelisted_tokens: old_contract.whitelisted_tokens,
            version: migration_version,
            withdraw_available: old_contract.withdraw_available,
            tip_available: old_contract.tip_available,
            generic_tips_available: false,

            telegram_tips_v1: old_contract.telegram_tips_v1,
            //total_chat_points: 0,
            chat_settings: LookupMap::new(StorageKey::ChatSettingsLookupMap),
            treasure: LookupMap::new(StorageKey::TreasureLookupMap),

            chat_points: LookupMap::new(StorageKey::ChatPointsLookupMapU128), // fix

            user_tokens_to_claim: LookupMap::new(StorageKey::UserTokensToClaimLookupMap),
            master_account_id: "zavodil.testnet".to_string(),
            linkdrop_account_id: "linkdrop.zavodil.testnet".to_string(),
            auth_account_id: "dev-1625611642901-32969379055293".to_string(),
            tiptoken_account_id: "tiptoken.zavodil.testnet".to_string(),
            total_tiptokens: 0,
            tiptokens_burned: 0
        }
    }

    // UPDATE BEFORE DEPLOYMENT
    /*
    const MASTER_ACCOUNT_ID: &str = "nearup_bot.app.near";
    const LINKDROP_ACCOUNT_ID: &str = "near";
    const AUTH_ACCOUNT_ID: &str = "auth.name.near";
    const TREASURE_ACCOUNT_ID: &str =


    const MASTER_ACCOUNT_ID: &str = "zavodil.testnet";
    const LINKDROP_ACCOUNT_ID: &str = "linkdrop.zavodil.testnet";
    const AUTH_ACCOUNT_ID: &str = "dev-1625611642901-32969379055293";
    */

    /*
  #[init(ignore_state)]
  #[allow(dead_code)]
  pub fn migrate_state_3_1() -> Self { // add telegram_users_in_chats, Migration to token balances / deposits
      let migration_version: u16 = 2;
      assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

      #[derive(BorshDeserialize)]
      struct OldContract {
          deposits: HashMap<AccountId, Balance>,
          telegram_tips: HashMap<String, Balance>,
          tips: UnorderedMap<AccountId, Vec<Tip>>,
          version: u16,
          withdraw_available: bool,
          tip_available: bool,
      }

      let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");
      let telegram_users_in_chats = LookupSet::new(StorageKey::TelegramUsersInChats);
      let tips_new = LookupMap::new(StorageKey::TipsLookupMap);
      let chat_points = LookupMap::new(StorageKey::ChatPointsLookupMap);

      let near_account_id: TokenAccountId = NEAR.to_string();
      let telegram_tips_new = LookupMap::new(b"a".to_vec());

      let mut deposits_new = LookupMap::new(StorageKey::TelegramDepositsLookupMap);
      for (account_id, deposit) in &old_contract.deposits {
          deposits_new.insert(
              &TokenByNearAccount {
                  account_id: account_id.to_string(),
                  token_account_id: near_account_id.clone(),
              },
              deposit,
          );
      }

      let mut whitelisted_tokens_new = LookupSet::new(StorageKey::WhitelistedTokensLookupSet);
      whitelisted_tokens_new.insert(&near_account_id);

      Self {
          deposits: deposits_new,
          telegram_tips: telegram_tips_new,
          tips: tips_new,
          telegram_users_in_chats,
          chat_points,
          whitelisted_tokens: whitelisted_tokens_new,
          version: migration_version,
          withdraw_available: old_contract.withdraw_available,
          tip_available: old_contract.tip_available,
          generic_tips_available: false,

          telegram_tips_v1: old_contract.telegram_tips
      }
  }

  #[init(ignore_state)]
  #[allow(dead_code)]
  pub fn migrate_state_3_2(iteration: u16) -> Self { // telegram_tips_v1 transition
      let migration_version: u16 = 3;
      assert_eq!(env::predecessor_account_id(), env::current_account_id(), "Private function");

      #[derive(BorshDeserialize)]
      struct OldContract {
          deposits: LookupMap<TokenByNearAccount, Balance>,
          telegram_tips: LookupMap<TokenByTelegramAccount, Balance>,
          tips: LookupMap<AccountId, Vec<Tip>>,
          telegram_users_in_chats: LookupSet<TelegramUserInChat>,
          chat_points: LookupMap<TelegramChatId, RewardPoint>,
          whitelisted_tokens: LookupSet<TokenAccountId>,
          version: u16,
          withdraw_available: bool,
          tip_available: bool,
          generic_tips_available: bool,

          telegram_tips_v1: HashMap<String, Balance>,
      }

      let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

      let near_account_id: TokenAccountId = NEAR.to_string();
      let mut telegram_tips_new =
          if iteration == 0 {
              LookupMap::new(StorageKey::TelegramTipsLookupMap)
          } else {
              old_contract.telegram_tips
          };
      let mut telegram_tips_v1 = old_contract.telegram_tips_v1.clone();
      let mut x: u32 = 1;

      for (telegram_account, amount) in &old_contract.telegram_tips_v1 {
          let telegram_id = telegram_account.parse::<u64>().unwrap_or(0);
          if telegram_id > 0 {
              telegram_tips_new.insert(
                  &TokenByTelegramAccount {
                      telegram_account: telegram_id,
                      token_account_id: near_account_id.clone(),
                  },
                  amount);

              telegram_tips_v1.remove(telegram_account);

              if x >= 250 {
                  break;
              }

              x += 1;
          } else {
              env::log(format!("Invalid telegram_account {}", telegram_account).as_bytes());
          }
      }

      env::log(format!("Pending items: {}", telegram_tips_v1.len()).as_bytes());


      Self {
          deposits: old_contract.deposits,
          telegram_tips: telegram_tips_new,
          tips: old_contract.tips,
          telegram_users_in_chats: old_contract.telegram_users_in_chats,
          chat_points: old_contract.chat_points,
          whitelisted_tokens: old_contract.whitelisted_tokens,
          version: migration_version,
          withdraw_available: old_contract.withdraw_available,
          tip_available: old_contract.tip_available,
          generic_tips_available: false,

          telegram_tips_v1
      }
  }
  */
}