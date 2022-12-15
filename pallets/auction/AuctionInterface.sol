// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Auction Precompile contract's address.
address constant AUCTION_PRECOMPILE_ADDRESS = 0x3333333330000000000000000000000000000000;

/// @dev The Auction Precompile contract's instance.
Auction constant AUCTION_CONTRACT = Auction(AUCTION_PRECOMPILE_ADDRESS);

/// @title  The Auction Precompile Interface
/// @dev The interface through which solidity contracts will interact with pallet-auction.
/// @custom:address 0x3333333330000000000000000000000000000000
interface Auction {
    /// @dev Gets the NFT of the specified listing.
    /// @custom:selector 60a08231
    /// @return A two uint256 parameters representing the listing item's class_id and token_id.
    function getListingItem(uint256 listing_id) external view returns (uint256,uint256);
    
    /// @dev Gets the meatverse of the specified listing.
    /// @custom:selector 60a08232
    /// @return An uint256 representing the listing item's metaverse_id.
    function getListingMetaverse(uint256 listing_id) external view returns (uint256);
    
    /// @dev Gets the currency and current price of the specified listing.
    /// @custom:selector 60a08233
    /// @return An uint256 representing the listing item's currency_id and value.
    function getListingPrice(uint256 listing_id) external view returns (uint256,uint256);
    
    /// @dev Gets the current end block of the specified listing.
    /// @custom:selector 60a08234
    /// @return An uint256 representing the listing item's end block.
    function getListingEndBlock(uint256 listing_id) external view returns (uint256);
   
    /// @dev Gets the current highest bidder address of the specified listing.
    /// @custom:selector 60a08235
    /// @return An uint256 representing the listing item's account address.
    function getListingHighestBidder(uint256 listing_id) external view returns (address);
    
    /// @dev Create auction for a specified asset
    /// @custom:selector a8159cb0
    /// @param owner address The address that creates the listing.
    /// @param class_id uint256 The class ID of the listed NFT.
    /// @param token_id uint256 The token ID of the listed NFT.
    /// @param end_block uint256 The end block of the auction lsiting.
    /// @param metaverse_id uint256 The ID of the metaverse where the item will be listed.
    /// @param value uint256 The minimum bid value for the auction listing.
    /// @return true if the auction was created, revert otherwise.
    function createAuction(address owner, uint256 class_id, uint256 token_id, uint256 end_block, uint256 metaverse_id, uint256 value) external returns (bool);
    
    /// @dev Bid for a specified listing
    /// @custom:selector a8159cb1
    /// @param bidder address The current bidder account.
    /// @param listing_id uint256 The ID of the auction listing.
    /// @param value uint256 The bid value for the auction listing.
    /// @return true if the bid was successful, revert otherwise.
    function bid(address bidder, uint256 listing_id, uint256 value) external returns (bool);
    
    /// @dev Finalize auction
    /// @custom:selector a8159cb2
    /// @param listing_id uint256 The ID of the auction listing.
    /// @return true if the auction was finalized, revert otherwise.
    function finalizeAuction(uint256 listing_id) external returns (bool);
    
    /// @dev Create buy now for a specified asset
    /// @custom:selector a8159cc0
    /// @param owner address The address that creates the listing.
    /// @param class_id uint256 The class ID of the listed NFT.
    /// @param token_id uint256 The token ID of the listed NFT.
    /// @param end_block uint256 The end block of the buy now lsiting.
    /// @param metaverse_id uint256 The ID of the metaverse where the item will be listed.
    /// @param value uint256 The value for the buy now listing.
    /// @return true if the buy now was created, revert otherwise.
    function createBuyNow(address owner, uint256 class_id, uint256 token_id, uint256 end_block, uint256 metaverse_id, uint256 value) external returns (bool);
    
    /// @dev Bid for a specified listing
    /// @custom:selector a8159cc1
    /// @param buyer address The buyer account's address.
    /// @param listing_id uint256 The ID of the buy now listing.
    /// @param value uint256 The value of the buy now listing.
    /// @return true if the buy now was successful, revert otherwise.
    function buyNow(address buyer, uint256 listing_id, uint256 value) external returns (bool);
    
    /// @dev Finalize auction
    /// @custom:selector a8159cc2
    /// @param owner address The address that created the listing.
    /// @param listing_id uint256 The ID of the listing.
    /// @return true if the listing was cancelled, revert otherwise.
    function cancelListing(address owner, uint256 listing_id) external returns (bool);
    
    /// @dev Make offer for a specified asset
    /// @custom:selector a8159cd0
    /// @param offeror address The address that creates the offer.
    /// @param class_id uint256 The class ID of the wanted NFT .
    /// @param token_id uint256 The token ID of the wanted NFT.
    /// @param value uint256 The offer value.
    /// @return true if the offer was made revert otherwise.
    function makeOffer(address offeror, uint256 class_id, uint256 token_id, uint256 value) external returns (bool);
    
    /// @dev Accept offer for a specified asset
    /// @custom:selector a8159cd1
    /// @param owner address The address that owns the wanted NFT.
    /// @param offeror address The address that created the offer.
    /// @param class_id uint256 The class ID of the wanted NFT .
    /// @param token_id uint256 The token ID of the wanted NFT.
    /// @return true if the offer was accepted, revert otherwise.
    function acceptOffer(address owner, address offeror, uint256 class_id, uint256 token_id) external returns (bool);
    
    /// @dev Withdraw offer for a specified asset
    /// @custom:selector a8159cd2
    /// @param offeror address The address that created the offer.
    /// @param class_id uint256 The class ID of the wanted NFT .
    /// @param token_id uint256 The token ID of the wanted NFT.
    /// @return true if the offer was withdrawn, revert otherwise.
    function withdrawOffer(address offeror, uint256 class_id, uint256 token_id) external returns (bool);
    
    /// @dev Event emited when an auction is created.
    /// @custom:selector edf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param listing_id uint256 The ID of the auction listing.
    /// @param class_id uint256 The class ID of the listed NFT.
    /// @param token_id uint256 The token ID of the listed NFT.
    /// @param end_block uint256 The end block of the auction lsiting.
    /// @param metaverse_id uint256 The ID of the metaverse where the item will be listed.
    /// @param value uint256 The minimum bid value for the auction listing.
    event AuctionCreated(uint256 listing_id, uint256 class_id, uint256 token_id, uint256 end_block, uint256 metaverse_id, uint256 value);
    
    /// @dev Event emited when a bid is submitted.
    /// @custom:selector ed1252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param bidder address The current bidder account.
    /// @param listing_id uint256 The ID of the auction listing.
    /// @param value uint256 The bid value for the auction listing.
    event BidSubmitted(address indexed bidder, uint256 listing_id, uint256 value);
    
    /// @dev Event emited when a auction is finalized manually.
    /// @custom:selector ed1352ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param listing_id uint256 The ID of the auction listing.
    event AuctionFinalized(uint256 listing_id);
    
    /// @dev Event emited when a buy now is created.
    /// @custom:selector eef252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param listing_id uint256 The ID of the auction listing.
    /// @param class_id uint256 The class ID of the listed NFT.
    /// @param token_id uint256 The token ID of the listed NFT.
    /// @param end_block uint256 The end block of the auction lsiting.
    /// @param metaverse_id uint256 The ID of the metaverse where the item will be listed.
    /// @param value uint256 The minimum bid value for the auction listing.
    event BuyNowCreated(uint256 listing_id, uint256 class_id, uint256 token_id, uint256 metaverse_id, uint256 end_block, uint256 value);
    
    /// @dev Event emited when a buy now lsiting offer is accepted.
    /// @custom:selector ed2252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param buyer address The buyer account of the listing.
    /// @param listing_id uint256 The ID of the buy now listing.
    /// @param value uint256 The value of the buy now listing.
    event BuyNowAccepted(address indexed buyer, uint256 listing_id, uint256 value);
    
    /// @dev Event emited when a listing is cancelled.
    /// @custom:selector ed1452ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param listing_id uint256 The ID of the listing.
    event ListingCancelled(uint256 listing_id);
    
    /// @dev Event emited when an offer for a NFT is made.
    /// @custom:selector fdf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param offeror address The address that created the offer.
    /// @param class_id uint256 The class ID of the NFT the offer was created for.
    /// @param token_id uint256 The token ID of the NFT the offer was created for.
    /// @param value uint256 The offer value.
    event OfferMade(address indexed offeror,  uint256 class_id, uint256 token_id, uint256 value);
    
    /// @dev Event emited when an offer for a NFT is accepted.
    /// @custom:selector fef252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param offeror address The address that created the offer.
    /// @param class_id uint256 The class ID of the NFT the offer was created for.
    /// @param token_id uint256 The token ID of the NFT the offer was created for.
    event OfferAccepted(address indexed offeror,  uint256 class_id, uint256 token_id);
    
    /// @dev Event emited when an offer for a NFT is withfrawn .
    /// @custom:selector fff252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param offeror address The address that created the offer.
    /// @param class_id uint256 The class ID of the NFT the offer was created for.
    /// @param token_id uint256 The token ID of the NFT the offer was created for.
    event OfferWithdrawn(address indexed offeror,  uint256 class_id, uint256 token_id);
}
