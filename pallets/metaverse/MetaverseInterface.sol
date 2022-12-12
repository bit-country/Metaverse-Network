// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Metaverse Precompile contract's address.
address constant METAVERSE_PRECOMPILE_ADDRESS = 0x1111111110000000000000000000000000000000;

/// @dev The Metaverse Precompile contract's instance.
Metaverse constant METAVERSE_CONTRACT = Meatverse(METAVERSE_PRECOMPILE_ADDRESS);

/// @title  The Metaverse Precompile Interface
/// @dev The interface through which solidity contracts will interact with pallet-meatverse.
/// @custom:address 0x1111111110000000000000000000000000000000
interface Metaverse {
    /// @dev Gets the metadata of the specified meatverse fund addreess.
    /// @custom:selector 70a08231
    /// @param owner The address that owns the token class.
    /// @param meatverse_id uint256 The ID of the metaverse.
    /// @return A bytes array representing the meatverse metadata.
    function getMetaverse(uint256 metaverse_id) external view returns (bytes);
    
    /// @dev Gets the balance of the specified meatverse fund addreess.
    /// @custom:selector 70a08231
    /// @param owner The address that owns the token class.
    /// @param meatverse_id uint256 The ID of the metaverse.
    /// @return An uint256 representing the class fund balance.
    function getMetaverseFundBalance(uint256 metaverse_id) external view returns (uint256);
    
    /// @dev Create metaverse for a specified address
    /// @custom:selector a9059cba
    /// @param owner address The address that will own the metaverse.
    /// @param metadata bytes The metadata of the meatverse.
    /// @return true if the mint was succesful, revert otherwise.
    function createMetaverse(address owner, bytes metadata) external returns (bool);
    
    /// @dev Withdraw from metaverse fund
    /// @custom:selector a1059cbb
    /// @param owner The meatverse owner address.
    /// @param meatverse_id uint256 The ID of the metaverse.
    /// @return true if the withdraw was succesful, revert otherwise.
    function withdrawFromMetaverseFund(address owner, uint256 metaverse_id) external returns (bool);
    
    /// @dev Event emited when a meatverse is created.
    /// @custom:selector ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param owner address The address that will own the metaverse.
    /// @param meatverse_id uint256 The ID of the metaverse.
    event MeatverseCreated(address indexed owner, uint256 meatverse_id);
    
    /// @dev Event emited when withdraw from meatverse fund is successful.
    /// @custom:selector adf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3cf
    /// @param owner The address sending the tokens
    /// @param meatverse_id uint256 The ID of the metaverse.
    /// @param balance uint256 The amount of tokens transfered.
    event MeatverseFundWithdraw(address indexed owner, uint256 meatverse_id, uint256 balance);
}


