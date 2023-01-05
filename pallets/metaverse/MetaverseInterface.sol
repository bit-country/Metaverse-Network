// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Metaverse Precompile contract's address.
address constant METAVERSE_PRECOMPILE_ADDRESS = 0x1111111110000000000000000000000000000000;

/// @dev The Metaverse Precompile contract's instance.
Metaverse constant METAVERSE_CONTRACT = Metaverse(METAVERSE_PRECOMPILE_ADDRESS);

/// @title  The Metaverse Precompile Interface
/// @dev The interface through which solidity contracts will interact with pallet-metaverse.
/// @custom:address 0x1111111110000000000000000000000000000000
interface Metaverse {
    /// @dev Gets the metadata of the specified metaverse fund address.
    /// @custom:selector 80a08231
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return A bytes array representing the metaverse metadata.
    function getMetaverse(uint256 metaverse_id) external view returns (bytes32);
    
    /// @dev Gets the balance of the specified metaverse fund address.
    /// @custom:selector 80a08232
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return An uint256 representing the class fund balance.
    function getMetaverseFundBalance(uint256 metaverse_id) external view returns (uint256);
    
    /// @dev Create metaverse for a specified address
    /// @custom:selector b9059cba
    /// @param owner address The address that will own the metaverse.
    /// @param metadata bytes The metadata of the metaverse.
    /// @return true if the mint was successful, revert otherwise.
    function createMetaverse(address owner, bytes32 metadata) external returns (bool);
    
    /// @dev Withdraw from metaverse fund
    /// @custom:selector b1059cbb
    /// @param owner The metaverse owner address.
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return true if the withdraw was successful, revert otherwise.
    function withdrawFromMetaverseFund(address owner, uint256 metaverse_id) external returns (bool);
    
    /// @dev Event emitted when a metaverse is created.
    /// @custom:selector edf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ff
    /// @param owner address The address that will own the metaverse.
    /// @param metaverse_id uint256 The ID of the metaverse.
    event metaverseCreated(address indexed owner, uint256 metaverse_id);
    
    /// @dev Event emitted when withdraw from metaverse fund is successful.
    /// @custom:selector bdf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3cf
    /// @param owner The address sending the tokens
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @param balance uint256 The amount of tokens transferred.
    event metaverseFundWithdraw(address indexed owner, uint256 metaverse_id, uint256 balance);
}
