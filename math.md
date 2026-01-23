# Math Example: Adding Liquidity

Let's trace a simplified example of a user adding liquidity to a single bin.

## Scenario
*   **Bin 100** (Price = 1.00 USDC per SOL)
*   User wants to add: **100 USDC** and **100 SOL**
*   **Initial State:** Bin is empty (`total_shares = 0`)

---

## 1. Input Distribution
User specifies:
*   `delta_id = 0` (Active Bin)
*   `dist_x = 5000` (50% of X)
*   `dist_y = 5000` (50% of Y)

## 2. Deposit Amount Calculation
Calculated from input amounts (e.g., total input 200 USDC, 200 SOL):

```math
deposit_x = amount_x \times \frac{dist_x}{10000} = 200 \times \frac{5000}{10000} = 100
```

```math
deposit_y = amount_y \times \frac{dist_y}{10000} = 200 \times \frac{5000}{10000} = 100
```

## 3. Shares Calculation (Minting)

### Case A: First Deposit (Bin Empty)
When `total_shares == 0`, we use the geometric mean:

```math
shares = \sqrt{deposit_x \times deposit_y}
```
```math
shares = \sqrt{100 \times 100} = 100
```
User receives **100 shares**.

**Bin State After Case A:**
*   `reserve_x = 100`
*   `reserve_y = 100`
*   `total_shares = 100`

---

### Case B: Subsequent Deposit
Now **User 2** comes along.
*   Wants to add **50 USDC** and **50 SOL**.
*   Bin already has liquidity (from Case A).

We calculate the share ratio for both X and Y and take the **minimum**:

```math
shares_x = deposit_x \times \frac{total\_shares}{reserve\_x} = 50 \times \frac{100}{100} = 50
```

```math
shares_y = deposit_y \times \frac{total\_shares}{reserve\_y} = 50 \times \frac{100}{100} = 50
```

```math
shares = \min(50, 50) = 50
```
User 2 receives **50 shares**.

**Bin State After Case B:**
*   `reserve_x = 100 + 50 = 150`
*   `reserve_y = 100 + 50 = 150`
*   `total_shares = 100 + 50 = 150`

---

### Why usage of `min`? (Preventing Dilution)
Imagine User 3 tries to cheat. They add **1000 USDC** but only **1 SOL**.
*   The bin ratio is 1:1 (150 X : 150 Y).
*   User 3 adds 10:1 ratio.

```math
shares_x = 1000 \times \frac{150}{150} = 1000
```

```math
shares_y = 1 \times \frac{150}{150} = 1
```

```math
shares = \min(1000, 1) = 1
```

They only get **1 share**.
*   Their extra 999 USDC is effectively "donated" to the pool (captured by existing LPs).
*   This forces LPs to add liquidity matching the current bin ratio.
