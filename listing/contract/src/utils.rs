use crate::*;

impl ListingAuction {
   pub(crate) fn assert_owner(&self) {
      assert_eq!(env::predecessor_account_id(), self.owner_id, "ERR_NO_ACCESS");
   }

   pub(crate) fn assert_auction_timestamp(&self, auction: &Auction) {
      let timestamp = env::block_timestamp();
      require!(timestamp >= auction.start_date && timestamp <= auction.end_date, "ERR_INACTIVE_EVENT");
   }

   pub(crate) fn assert_auction_end_date_already_passed(&self, auction: &Auction) {
      let timestamp = env::block_timestamp();
      require!(timestamp > auction.end_date, "ERR_END_DATE_DIDNT_PASS");
   }

   pub(crate) fn assert_auction_unlock_date_for_winner_already_passed(&self, auction: &Auction) {
      let timestamp = env::block_timestamp();
      require!(timestamp > auction.unlock_date_for_winner, "ERR_UNLOCK_DATE_DIDNT_PASS");
   }

   pub(crate) fn assert_tiptoken(&self, token_id: TokenId) {
      assert_eq!(token_id, self.tiptoken_account_id, "ERR_WRONG_TOKEN")
   }

   // if token exists in action
   pub(crate) fn assert_auction_token(&self, auction: &Auction, token_id: &TokenId) {
      require!(auction.tokens.contains(&token_id), "ERR_TOKEN_WAS_NOT_ADDED_TO_AUCTION");
   }
}

pub(crate) fn unordered_map_pagination<K, VV, V>(
   m: &UnorderedMap<K, VV>,
   from_index: Option<u64>,
   limit: Option<u64>,
) -> Vec<(K, V)>
   where
      K: BorshSerialize + BorshDeserialize,
      VV: BorshSerialize + BorshDeserialize,
      V: From<VV>,
{
   let keys = m.keys_as_vector();
   let values = m.values_as_vector();
   let from_index = from_index.unwrap_or(0);
   let limit = limit.unwrap_or(keys.len());
   (from_index..std::cmp::min(keys.len(), from_index + limit))
      .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap().into()))
      .collect()
}
