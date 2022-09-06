use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Auction {
    pub start_date: Timestamp,
    pub end_date: Timestamp,
    pub unlock_date_for_winner: Timestamp,

    // token partitions
    pub tokens: UnorderedSet<TokenId>,
    // deposit of every auction participant
    pub deposits: UnorderedMap<AuctionDeposit, Balance>,
    // rewards in token distributed to winner supporter, by every reward provider
    pub rewards: UnorderedMap<AuctionDeposit, Balance>,
    // users who received their rewards
    pub reward_receivers: UnorderedSet<AccountId>,

    // total deposits of every auction participant
    pub total_deposits: UnorderedMap<TokenId, Balance>,
    // total rewards of every auction participant
    pub total_rewards: UnorderedMap<TokenId, Balance>,

    // winner (after the finalization)
    pub winner_id: Option<TokenId>,
}

impl Auction {
    pub fn auction_total_rewards_list(&self) -> Vec<(TokenId, WrappedBalance)> {
        self
            .total_rewards
            .iter()
            .map(|(token_id, balance)| (token_id, balance.into()))
            .collect()
    }

    pub fn auction_total_deposits_list(&self) -> Vec<(TokenId, WrappedBalance)> {
        self
            .total_deposits
            .iter()
            .map(|(token_id, balance)| (token_id, balance.into()))
            .collect()
    }

    pub fn deposits_list(&self) -> Vec<(TokenId, AccountId, WrappedBalance)> {
        self
            .deposits
            .iter()
            .map(|(deposit, balance)| (deposit.token_id, deposit.account_id, balance.into()))
            .collect()
    }

    pub fn rewards_list(&self) -> Vec<(TokenId, AccountId, WrappedBalance)> {
        self
            .rewards
            .iter()
            .map(|(deposit, balance)| (deposit.token_id, deposit.account_id, balance.into()))
            .collect()
    }
}

#[near_bindgen]
impl ListingAuction {
    pub fn set_auction_end_date(&mut self, auction_id: AuctionId, end_date: Timestamp) {
        self.assert_owner();
        let mut auction: Auction = self.unwrap_auction(&auction_id);
        auction.end_date = end_date;
        self.auctions.insert(&auction_id, &VAuction::Current(auction));
    }

    pub fn set_auction_unlock_date_for_winner(&mut self, auction_id: AuctionId, unlock_date_for_winner: Timestamp) {
        self.assert_owner();
        let mut auction: Auction = self.unwrap_auction(&auction_id);
        auction.unlock_date_for_winner = unlock_date_for_winner;
        self.auctions.insert(&auction_id, &VAuction::Current(auction));
    }

    pub fn add_listing_auction(&mut self,
                               start_date: Option<Timestamp>,
                               end_date: Timestamp,
                               unlock_date_for_winner: Timestamp) -> AuctionId {
        self.assert_owner();

        let start_date = if let Some(start_date) = start_date {
            start_date
        } else {
            env::block_timestamp()
        };

        let auction: Auction = Auction {
            start_date,
            end_date,
            unlock_date_for_winner,
            tokens: UnorderedSet::new(StorageKey::AuctionTokens { auction_id: self.next_auction_id }),
            deposits: UnorderedMap::new(StorageKey::AuctionDeposits { auction_id: self.next_auction_id }),
            rewards: UnorderedMap::new(StorageKey::AuctionRewards { auction_id: self.next_auction_id }),
            total_deposits: UnorderedMap::new(StorageKey::AuctionTotalDeposits { auction_id: self.next_auction_id }),
            total_rewards: UnorderedMap::new(StorageKey::AuctionTotalRewards { auction_id: self.next_auction_id }),
            reward_receivers: UnorderedSet::new(StorageKey::AuctionRewardReceivers { auction_id: self.next_auction_id }),
            winner_id: None,
        };

        let timestamp = env::block_timestamp();
        assert!(timestamp <= auction.start_date, "ERR_START_DATE_NOT_PASSED");
        assert!(auction.end_date > auction.start_date, "ERR_WRONG_END_DATE");
        assert!(auction.unlock_date_for_winner > auction.end_date, "ERR_WRONG_UNLOCK_DATE");

        self.auctions.insert(&self.next_auction_id, &VAuction::Current(auction));


        self.active_auctions.insert(&self.next_auction_id);

        self.next_auction_id += 1;

        self.next_auction_id
    }

    pub fn get_auction(&self, auction_id: AuctionId) -> AuctionOutput {
        self.auctions.get(&auction_id).expect("ERR_NO_AUCTION").into()
    }

    pub fn get_auction_tokens(&self, auction_id: AuctionId) -> Vec<TokenId> {
        self.unwrap_auction(&auction_id).tokens.to_vec()
    }

    pub fn get_auction_tokens_number(&self, auction_id: AuctionId) -> u64 {
        self.unwrap_auction(&auction_id).tokens.len()
    }

    pub fn get_auction_total_deposits(&self, auction_id: AuctionId) -> Vec<(TokenId, WrappedBalance)> {
        self.unwrap_auction(&auction_id).auction_total_deposits_list()
    }

    pub fn get_auction_total_rewards(&self, auction_id: AuctionId) -> Vec<(TokenId, WrappedBalance)> {
        self.unwrap_auction(&auction_id).auction_total_rewards_list()
    }

    pub fn get_auction_total_deposit(&self, auction_id: AuctionId, token_id: TokenId) -> WrappedBalance {
        self.unwrap_auction(&auction_id).total_deposits.get(&token_id).unwrap_or_default().into()
    }

    pub fn get_auction_total_reward(&self, auction_id: AuctionId, token_id: TokenId) -> WrappedBalance {
        self.unwrap_auction(&auction_id).total_rewards.get(&token_id).unwrap_or_default().into()
    }

    pub fn get_auction_winner(&self, auction_id: AuctionId) -> Option<TokenId> {
        self.unwrap_auction(&auction_id).winner_id
    }

    pub fn get_user_deposit(&self, auction_id: AuctionId, account_id: AccountId, token_id: TokenId) -> WrappedBalance {
        self.unwrap_auction(&auction_id).deposits.get(&AuctionDeposit {
            account_id,
            token_id,
        }).unwrap_or(0).into()
    }
}

impl ListingAuction {
    pub(crate) fn unwrap_auction(&self, auction_id: &AuctionId) -> Auction {
        self.auctions.get(auction_id).expect("ERR_NO_AUCTION").into()
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAuction {
    Current(Auction),
}

impl From<VAuction> for Auction {
    fn from(v_auction: VAuction) -> Self {
        match v_auction {
            VAuction::Current(auction) => auction,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AuctionOutput {
    pub start_date: Timestamp,
    pub end_date: Timestamp,
    pub unlock_date_for_winner: Timestamp,
    pub rewards: Vec<(TokenId, AccountId, WrappedBalance)>,
    pub deposits: Vec<(TokenId, AccountId, WrappedBalance)>,
    pub total_rewards: Vec<(TokenId, WrappedBalance)>,
    pub total_deposits: Vec<(TokenId, WrappedBalance)>,
    pub reward_received_by: Vec<AccountId>,
    pub winner_id: Option<AccountId>,
}

impl From<VAuction> for AuctionOutput {
    fn from(v_auction: VAuction) -> Self {
        match v_auction {
            VAuction::Current(auction) => AuctionOutput {
                start_date: auction.start_date,
                end_date: auction.end_date,
                unlock_date_for_winner: auction.unlock_date_for_winner,

                rewards: auction.rewards_list(),
                deposits: auction.deposits_list(),
                total_rewards: auction.auction_total_rewards_list(),
                total_deposits: auction.auction_total_deposits_list(),

                reward_received_by: auction.reward_receivers.to_vec(),

                winner_id: auction.winner_id,
            }
        }
    }
}
