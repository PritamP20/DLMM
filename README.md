# DLMM (Dynamic Liquidity Market Maker)

A Solana-based concentrated liquidity AMM implementation using Anchor framework.

---

## How DLMM Works

### 1. Core Concept: Concentrated Liquidity in Bins

Unlike traditional AMMs (like Uniswap V2) where liquidity is spread across the entire price curve (0 to ∞), DLMM **concentrates liquidity into discrete price bins**.

```
Traditional AMM:     DLMM:
                     
Price ∞ ─────────   Price ∞ ─────────
        │░░░░░░│            │        │
        │░░░░░░│            │        │
        │░░░░░░│            │████████│ ← Bin 102 (price $1.02)
        │░░░░░░│            │████████│ ← Bin 101 (price $1.01)  
        │░░░░░░│            │████████│ ← Bin 100 (Active, price $1.00)
        │░░░░░░│            │████████│ ← Bin 99 (price $0.99)
        │░░░░░░│            │        │
Price 0 ─────────   Price 0 ─────────
  (Spread thin)       (Concentrated)
```

---

### 2. Bin Structure

Each **bin** represents a specific price point and holds:
- `reserve_x` - Amount of Token X
- `reserve_y` - Amount of Token Y
- `total_shares` - LP shares issued for this bin
- `bin_id` - Unique identifier (determines price)

**Price Formula:**
```
price(bin_id) = (1 + bin_step/10000) ^ (bin_id - offset)
```

Example with `bin_step = 100` (1%):
- Bin 100 → Price 1.00
- Bin 101 → Price 1.01
- Bin 102 → Price 1.0201

---

### 3. Active Bin

The **active bin** is where swaps happen. It contains BOTH tokens:

```
        Token Y only
           ↑
   Bin 103 │████████│  ← Only Y (above active)
   Bin 102 │████████│  ← Only Y
   Bin 101 │██████░░│  ← ACTIVE BIN (has X and Y)
   Bin 100 │████████│  ← Only X (below active)
   Bin 99  │████████│  ← Only X
           ↓
        Token X only
```

**Key insight:**
- Bins **above** active bin: Only Token Y
- Bins **below** active bin: Only Token X
- **Active bin**: Contains both (swap happens here)

---

### 4. How Swaps Work

When swapping X → Y:

1. **Consume Y from active bin** until it runs out
2. **Active bin moves up** (price increases)
3. **Continue consuming Y** from next bin

```
Before Swap:              After Swap (price moved up):
                          
Bin 102 │░░░░░░│          Bin 102 │██████│ ← New active
Bin 101 │██░░░░│ Active → Bin 101 │██████│ ← Now only X
Bin 100 │██████│          Bin 100 │██████│
```

---

### 5. Adding Liquidity

When you add liquidity, you specify:
- **Which bins** to deposit into (relative to active bin)
- **How much** X and Y to distribute

**Example:**
```
Add 1000 USDC, 1 SOL to bins [-1, 0, +1] around active bin

Bin Distribution:
- Bin -1 (below active): 100% USDC, 0% SOL
- Bin 0 (active):        50% USDC, 50% SOL  
- Bin +1 (above active): 0% USDC, 100% SOL
```

**Share Calculation:**
- First deposit to empty bin: `shares = √(deposit_x × deposit_y)`
- Subsequent deposits: `shares = deposit_amount × total_shares / bin_reserves`

---

### 6. Why DLMM is Better

| Feature | Traditional AMM | DLMM |
|---------|----------------|------|
| Capital Efficiency | Low (spread thin) | High (concentrated) |
| Slippage | Higher | Lower |
| LP Earnings | Lower | Higher (more trades at your price) |
| Impermanent Loss | Always | Only in active bin |

---

## Contract Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      initialize_lbpair                       │
│  Creates: LbPair (pool state with token mints, bin_step)    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                     initialize_bin_array                     │
│  Creates: BinArray (70 bins per array, indexed)             │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                       add_liquidity                          │
│  1. User specifies: amount_x, amount_y, bin_distribution    │
│  2. For each bin in distribution:                           │
│     - Calculate deposit amounts (based on distribution %)    │
│     - Calculate shares to mint                               │
│     - Update bin.reserve_x, bin.reserve_y, bin.total_shares │
│     - Update position.liquidity_shares[bin_index]           │
│  3. Transfer tokens from user to reserves                   │
│  4. Update lb_pair total reserves                           │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                          swap                                │
│  1. Find active bin                                          │
│  2. Trade against active bin reserves                        │
│  3. If bin exhausted, move to next bin                       │
│  4. Collect fees into fee_x/y_per_share                      │
└─────────────────────────────────────────────────────────────┘
```

---

## State Accounts

### LbPair
The main pool state containing token mints, reserves, bin configuration, and fee parameters.

### BinArray
Groups of 70 bins stored together for efficiency. Each bin tracks its reserves and LP shares.

### Position
Tracks a user's liquidity shares across bins they've deposited into.

---

## Building

```bash
anchor build
```

## Testing

```bash
anchor test
```

---

## License

MIT
