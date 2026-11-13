# 6. Economy

## 6.1 Principles

1. Players gather and make most goods.
2. Geography is important. Prices are different in each region.
3. Each coin and each item has a known source and a known sink.
4. The game never makes or destroys coins or items by mistake. Tests check this rule (see 6.10).

## 6.2 Items

Each item type has a definition in a RON file. The definition contains:

- A stable ID, for example `item.log.oak`.
- A name key for localization.
- A category, a stack size and a base value.
- A storage class: personal or bulk (see 6.3).
- Trade rules: tradeable, or bound to the character.
- For gear: the durability, the slot and the skill requirements.

Each item that does not stack has a unique instance ID. Items that stack move as quantities.

## 6.3 Storage

| Storage | Contents | Access |
|---|---|---|
| Pack | 28 slots | The character carries the pack. |
| Vault | Coins and personal items | Linked. A player can open the same vault in each city. |
| Warehouse | Bulk goods | Local. Each city has a separate warehouse for each player. |

Personal items are gear, tools, small stacks of consumables, quest items and valuables.

Bulk goods are raw materials and trade goods, for example ore, logs, planks, cloth bolts, grain and spices. Bulk goods stay in the warehouse of the city where the player stored them. A player moves bulk goods in one of these ways:

- Carry them in the pack, on foot.
- Use a pack animal (a tamed creature).
- Hire a caravan.
- Pay a courier.

Bulk goods cannot go through a waystone or with a teleport charm. This rule keeps the regional prices.

## 6.4 Regional exchanges

Each major city has an exchange with its own order book.

- A player places buy orders and sell orders with a limit price.
- The exchange matches orders by price first, then by time.
- The goods and the coins of an order settle in the warehouse and the vault of that city.
- Each sale has a tax of 2%. The tax is a coin sink.
- Each player has a limit for the number of open orders. The Merchanting level increases the limit.
- The exchange shows the price history.
- All shards share the same exchanges.

NPC traders place a small number of orders at stable prices for basic goods. These orders give liquidity when few players trade. They also set a floor price and a ceiling price for basic goods.

## 6.5 Direct trade

Two players near each other can trade directly:

1. One player sends a trade request. The other player accepts it.
2. Both players put items and coins in the trade window.
3. Both players accept. Each change to the window cancels both accepts.
4. A second screen shows the final contents. Both players confirm.
5. The server moves all items and coins in one database transaction.

A new account cannot trade coins for the first 10 hours of play. This rule makes real-money trade and bot farms less useful.

## 6.6 NPC shops

NPC shops sell basic tools, starter gear and supplies. Each shop has a limited stock that fills again over time. Shops buy junk items and treasure items for a low price.

## 6.7 Transport

| Method | Speed | Cost | Risk |
|---|---|---|---|
| Pack on foot | Fast, for small loads | None | Low |
| Pack animal | Walk speed, 40 slots | Animal upkeep | Low |
| Caravan | Slow, for large loads | A hire fee | Bandits on some roads. High in the Frontier. |
| Courier | 30 minutes or more | 5% to 15% of the value of the goods, from the distance | None |

Players with the Merchanting skill can lead caravans and accept courier contracts from other players.

## 6.8 Crafting

- Each recipe has a RON definition: inputs, outputs, tools, station, skill level and XP.
- A station is a fixed object, for example a forge, an anvil, a loom or a kitchen.
- A higher skill level gives a better chance of high quality.
- A player can craft a batch. Each item in the batch takes time.

## 6.9 Sources and sinks

Coin sources:

- Coins from defeated enemies.
- Rewards from quests and tasks.
- NPC shops that buy junk items and treasure items.

Coin sinks:

- The exchange tax.
- Waystone fees.
- Repairs.
- Courier fees and caravan fees.
- Trainer fees.
- NPC goods, for example dyes and Starweaving reagents.
- Building costs (later milestone).

Item sinks:

- Gear durability, and repairs that use materials.
- Consumables: food, potions and charms.
- Crafting inputs.

The server records metrics for the total coins, the coins from each source and the coins to each sink. A dashboard shows these metrics.

## 6.10 Integrity

- Each change to items or coins is one database transaction.
- An append-only ledger records each change.
- Property tests check that trade, the exchanges, couriers and crafting do not make or destroy coins or items. Only the declared sources and sinks can do this.
