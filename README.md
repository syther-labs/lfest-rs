# Leveraged Futures Exchange for Simulated Trading (LFEST)
:warning: This is a personal project, use a your own risk. 

:warning: The results may not represent real trading results on any given exchange. 

lfest-rs is a blazingly fast simulated exchange capable of leveraged positions.
 It gets fed external bid ask data to update the internal state
  and check for order execution. For simplicity's sake (and performance) the exchange does not use an order book.
  Supported futures types are both linear and inverse futures.

### Order Types
The supported order types are:
- market        - aggressively execute against the best bid / ask
- limit         - passively place an order into the orderbook

### Performance Metrics:
The following performance metrics are available through AccTracker struct:
- win_ratio
- profit_loss_ratio
- total_rpnl
- sharpe
- sortino
- cumulative fees
- max_drawdown_wallet_balance
- max_drawdown_total
- num_trades
- turnover
- trade_percentage
- buy_ratio
- limit_order_fill_ratio
- limit_order_cancellation_ratio
- historical_value_at_risk
- cornish_fisher_value_at_risk
- d_ratio

Some of these metric may behave differently from what you would expect, so make sure to take a look at the code.

### How to use
To use this crate in your project, add the following to your Cargo.toml:
```
[dependencies]
lfest = "0.24.0"
```

Then proceed to use it in your code.
For an example see [examples](examples/basic.rs)

### TODOs:
- proper liquidations
- add order filter configuration such as min_qty and qty_precision
- add max_num_limit_orders to config
- impl Display for Side and FuturesType
- add optional order filtering such as
  * PriceFilters:
    * min_price
    * max_price
    * tick_size
  * SizeFilters:
    * min_size
    * max_size
    * step_size
- add config option to disable acc_tracker, which will save a bunch of RAM

### Contributions
If you find a bug or would like to help out, feel free to create a pull-request.

### Donations :moneybag: :money_with_wings:
I you would like to support the development of this crate, feel free to send over a donation:

Monero (XMR) address:
```plain
47xMvxNKsCKMt2owkDuN1Bci2KMiqGrAFCQFSLijWLs49ua67222Wu3LZryyopDVPYgYmAnYkSZSz9ZW2buaDwdyKTWGwwb
```

![monero](img/monero_donations_qrcode.png)

### License
Copyright (C) 2020  <Mathis Wellmann wellmannmathis@gmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

![GNU AGPLv3](img/agplv3.png)
