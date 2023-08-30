// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The NFT Precompile contract's address.
address constant NFT_PRECOMPILE_ADDRESS = 0x0202020202020202020000000000000000000000;

/// @dev The NFT Precompile contract's instance.
NFT constant NFT_CONTRACT = NFT(NFT_PRECOMPILE_ADDRESS);

/// @title  The NFT Precompile Interface
/// @dev The interface through which solidity contracts will interact with pallet-nft.
/// @custom:address 0x0202020202020202020000000000000000000000
interface NFT {
    /// @dev Gets the NFT token metadata.
    /// @param class_id The class ID of the NFT.
    /// @param token_id The token ID of the NFT.
    /// @return A bytes array representing the NFT metadata.
    function getNftMetadata(uint256 class_id, uint256 token_id) external view returns (bytes32);

    /// @dev Gets the NFT token address.
    /// @param class_id The class ID of the NFT.
    /// @param token_id The token ID of the NFT.
    /// @return An address representing the NFT token address.
    function getNftAddress(uint256 class_id, uint256 token_id) external view returns (address);

    /// @dev Gets the NFT token owner.
    /// @param class_id The class ID of the NFT.
    /// @param token_id The token ID of the NFT.
    /// @return An address representing the NFT owner.
    function getAssetOwner(uint256 class_id, uint256 token_id) external view returns (address);

    /// @dev Gets the balance of the specified class fund addreess.
    /// @param class_owner The address that owns the token class.
    /// @param class_id The class ID of the class fund.
    /// @return An uint256 representing the class fund balance.
    function getClassFundBalance(address class_owner, uint256 class_id) external view returns (uint256);
    
    /// @dev Mint token for a specified address
    /// @param owner address The address that will mint the token class.
    /// @param metadata bytes The metadata of the class.
    /// @param collection_id unit256 The colection ID of the token class.
    /// @param royalty_fee unit256 The royalty fee for the new token class. 
    /// @param mint_limit unit256 The maximum number of tokens that can be minted for this class.
    /// @return true if the mint was succesful, revert otherwise.
    function createClass(address owner, bytes32 metadata, uint256 collection_id, uint256 royalty_fee, uint256 mint_limit) external returns (bool);
    
    /// @dev Mint token for a specified address
    /// @param owner address The address that will mint the tokens.
    /// @param class_id uint256 The class ID of the tokens.
    /// @param metadata bytes The metadata of tokens.
    /// @param quantity unit256 The amount of tokens that will be minted.
    /// @return true if the mint was succesful, revert otherwise.
    function mintNfts(address owner, uint256 class_id, bytes32 metadata, uint256 quantity) external returns (bool);

    /// @dev Transfer token for a specified address
    /// @param to The address to transfer to.
    /// @param class_id uint256 The class ID of the NFT that will be transferred.
    /// @param token_id uint256 The token ID of the NFT that will be transferred.
    /// @return true if the transfer was succesful, revert otherwise.
    function transferNft(address to, uint256 class_id, uint256 token_id) external returns (bool);
    
    /// @dev Burn token for a specified address
    /// @param owner The owner of the NFT
    /// @param class_id uint256 The class ID of the NFT that will be burned.
    /// @param token_id uint256 The token ID of the NFT that will be burned.
    /// @return true if the burn was succesful, revert otherwise.
    function burnNft(address owner, uint256 class_id, uint256 token_id) external returns (bool);
    
    
    /// @dev Withdraw funds from class fund
    /// @param owner The address to transfer to.
    /// @param class_id The class ID of the class fund.
    /// @return true if the burn was succesful, revert otherwise.
    function withdrawFromClassFund(address owner, uint256 class_id) external returns (bool);
    
    /// @dev Event emited when a class is created.
    /// @param owner address The address that creates the class.
    /// @param class_id uint256 The new class ID.
    event ClassCreated(address indexed owner, uint256 class_id);
    
    /// @dev Event emited when a mint has been performed.
    /// @param owner address The address that will mint the tokens.
    /// @param class_id uint256 The class ID of the tokens.
    /// @param quantity unit256 The amount of tokens that will are minted.
    /// @param token_id unit256 The last minted token ID.
    event Mint(address indexed owner, uint256 class_id, uint256 quantity, uint256 token_id);

    /// @dev Event emited when a transfer has been performed.
    /// @param from address The address sending the tokens
    /// @param to address The address receiving the tokens.
    /// @param class_id uint256 The class ID of the NFT that will be trnasferred.
    /// @param token_id uint256 The token ID of the NFT that will be trnasferred.
    event Transfer(address indexed from, address indexed to, uint256 class_id, uint256 token_id);
    
    /// @dev Event emited when a burn has been performed.
    /// @param owner The owner of the NFT
    /// @param class_id uint256 The class ID of the NFT that will be burned.
    /// @param token_id uint256 The token ID of the NFT that will be burned.
    event Burn(address indexed owner, uint256 class_id, uint256 token_id);
    
    /// @dev Event emited when a withdraw from.
    /// @param owner The class owner.
    /// @param class_id uint256 The class ID of the fund.
    /// @param balance uint256 The amount of tokens withdrawn.
    event ClassFundWithdraw(address indexed owner, uint256 class_id, uint256 balance);
}
