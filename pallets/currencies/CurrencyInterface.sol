// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Currency Precompile contract's address.
address constant CURRENCY_PRECOMPILE_ADDRESS = 0x0000000000000000000000000000000000000000;

/// @dev The Currency Precompile contract's instance.
Currency constant CURRENCY_CONTRACT = Currency(CURRENCY_PRECOMPILE_ADDRESS);

/// @title The Currency Precompile Interface
/// @dev The interface through which solidity contracts will interact with pallet-currency.
/// @custom:address 0x0000000000000000000000000000000000000000
interface Currency {
    /// @dev Gets the total supply of a currency.
    /// @custom:selector 18160ddd             
    /// @return An uint256 representing the total supply of a currency.
    function totalSupply() external view returns (uint256);

    /// @dev Gets balance of an address.
    /// @custom:selector 70a08231
    /// @param owner address The address that owns a currency.
    /// @return An uint256 representing the balance of the owner.
    function balanceOf(address owner) external view returns (uint256);
    
    /// @dev Gets the currency  allowance of an address.
     /// @custom:selector dd62ed3e
    /// @param owner address The address that owns a currency
    /// @param currency address The currency address
    /// @return An uint256 representing of the currency for the owner.
    function allowance(address owner, address currency) external view returns (uint256);

    /// @dev Gets the name of a currency.
    /// @custom:selector 06fdde03 
    /// @return A bytes32 array representing the name of a currency.
    function name() external view returns (bytes32);

    /// @dev Gets the symbol of a currency.
    /// @custom:selector 95d89b41
    /// @return A bytes32 array representing the symbol of a currency.
    function symbol() external view returns (bytes32);

    /// @dev Gets the decimals of a currency.
    /// @custom:selector 313ce567
    /// @return An uint256 representing the decimals of a currency.
    function decimals() external view returns (uint256);
    
    /// @dev Transfer currency to a specified address
    /// @custom:selector a9059cbb
    /// @param receiver address The address that will receive the currency.
    /// @param value uint256 The value that will be transferred.
    /// @return true if the transfer was successful, revert otherwise
    function transfer(address receiver, uint256 value) external returns (bool);
    
    /// @dev Approve currency for transfer.
    /// @custom:selector 095ea7b3
    /// @param owner The currency owner address.
    /// @param value uint256 The value that will be approved.
    /// @return true if the approval was successful, revert otherwise.
    function approve(address spender, uint256 value) external returns (bool);

    /// @dev Transfer currency from a specified address to another one.
    /// @custom:selector 23b872dd
    /// @param sender The currency sender address.
    /// @param receiver The currency receiver address.
    /// @param value uint256 The value that will be transferred.
    /// @return true if the transfer was successful, revert otherwise.
    function transferFrom(address sender, address receiver, uint256 value) external returns (bool);
    
    /// @dev Event emitted when a currency is transferred.
    /// @custom:selector ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
    /// @param sender address The address that transferred the currency.
    /// @param receiver address The address that received the currency.
    /// @param value uint256 The value that was transferred.
    event Transferred(address indexed sender, address receiver, uint256 value);
    
    /// @dev Event emitted when a currency is approved.
    /// @custom:selector 8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
    /// @param owner The currency owner address.
    /// @param value uint256 The value that was approved.
    event Approved(address indexed owner, address indexed spender, uint256 value);
}
