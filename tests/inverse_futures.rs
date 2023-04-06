//! Test file for the inverse futures mode of the exchange

use fpdec::{Dec, Decimal};
use lfest::{account_tracker::NoAccountTracker, prelude::*};

#[test]
fn inv_long_market_win_full() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter::default(),
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(999.0), quote!(1000.0)))
        .unwrap();

    let value: BaseCurrency = exchange.account().margin().available_balance() * base!(0.8);
    let size = value.convert(exchange.ask());
    let o = Order::market(Side::Buy, size).unwrap();
    exchange.submit_order(o).unwrap();

    let _ = exchange
        .update_state(1, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    let fee_quote = size.fee_portion(fee_taker);
    let fee_base1 = fee_quote.convert(exchange.bid());

    assert_eq!(exchange.account().position().size(), size);
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        base!(1.0) - fee_base1
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.8));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(0.2) - fee_base1)
    );

    let _ = exchange
        .update_state(1, bba!(quote!(2000.0), quote!(2001.0)))
        .unwrap();
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.4));

    let size = quote!(800.0);
    let fee_base2 = size.fee_portion(fee_taker);
    let fee_asset2 = fee_base2.convert(quote!(2000.0));

    let o = Order::market(Side::Sell, size).unwrap();
    exchange.submit_order(o).unwrap();

    assert_eq!(exchange.account().position().size(), quote!(0.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        (base!(1.4) - fee_base1 - fee_asset2)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.0));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(1.4) - fee_base1 - fee_asset2)
    );
}

#[test]
fn inv_long_market_loss_full() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter::default(),
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(999.0), quote!(1000.0)))
        .unwrap();

    let o = Order::market(Side::Buy, quote!(800.0)).unwrap();
    exchange.submit_order(o).unwrap();

    assert_eq!(exchange.account().position().size(), quote!(800.0));
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(exchange.account().margin().wallet_balance(), base!(0.99952));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(0.19952)
    );
    assert_eq!(exchange.account().margin().order_margin(), base!(0.0));
    assert_eq!(exchange.account().margin().position_margin(), base!(0.8));

    let _ = exchange
        .update_state(2, bba!(quote!(800.0), quote!(801.0)))
        .unwrap();
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(-0.2));

    let size = quote!(800.0);
    let o = Order::market(Side::Sell, size).unwrap();
    exchange.submit_order(o).unwrap();

    let fee_quote0 = size.fee_portion(fee_taker);
    let fee_base0 = fee_quote0.convert(quote!(1000.0));

    let fee_quote1 = size.fee_portion(fee_taker);
    let fee_base1 = fee_quote1.convert(quote!(800.0));

    let fee_combined = fee_base0 + fee_base1;

    assert_eq!(exchange.account().position().size(), quote!(0.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        (base!(0.8) - fee_combined)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.0));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(0.8) - fee_combined)
    );
}

#[test]
fn inv_short_market_win_full() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter::default(),
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    let o = Order::market(Side::Sell, quote!(800.0)).unwrap();
    exchange.submit_order(o).unwrap();
    let _ = exchange
        .update_state(1, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    assert_eq!(exchange.account().position().size(), quote!(-800.0));

    let _ = exchange
        .update_state(1, bba!(quote!(799.0), quote!(800.0)))
        .unwrap();
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.2));

    let size = quote!(800.0);
    let o = Order::market(Side::Buy, size).unwrap();
    let order_err = exchange.submit_order(o);
    assert!(order_err.is_ok());
    let _ = exchange
        .update_state(2, bba!(quote!(799.0), quote!(800.0)))
        .unwrap();

    let fee_quote0 = size.fee_portion(fee_taker);
    let fee_base0 = fee_quote0.convert(quote!(1000.0));

    let fee_quote1 = size.fee_portion(fee_taker);
    let fee_base1 = fee_quote1.convert(quote!(800.0));

    let fee_combined: BaseCurrency = fee_base0 + fee_base1;

    assert_eq!(exchange.account().position().size(), quote!(0.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        base!(1.2) - fee_combined
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.0));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(1.2) - fee_combined
    );
}

#[test]
fn inv_short_market_loss_full() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter {
            min_quantity: quote!(0),
            max_quantity: quote!(0),
            step_size: quote!(0.1),
        },
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(1000), quote!(1001)))
        .unwrap();

    let value: BaseCurrency = BaseCurrency::new(Dec!(0.4));
    let size = value.convert(exchange.bid());
    let o = Order::market(Side::Sell, size).unwrap();
    exchange.submit_order(o).unwrap();

    let _ = exchange
        .update_state(1, bba!(quote!(999), quote!(1000)))
        .unwrap();

    let fee_quote1 = size.fee_portion(fee_taker);
    let fee_base1 = fee_quote1.convert(quote!(1000));

    assert_eq!(exchange.account().position().size(), size.into_negative());
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        base!(1.0) - fee_base1
    );
    assert_eq!(
        exchange.account().margin().position_margin().inner(),
        Dec!(0.4)
    );
    assert_eq!(
        exchange.account().margin().available_balance().inner(),
        Dec!(0.59976)
    );

    let _ = exchange
        .update_state(2, bba!(quote!(1999), quote!(2000)))
        .unwrap();

    let size = quote!(400.0);
    let fee_quote2 = size.fee_portion(fee_taker);
    let fee_base2: BaseCurrency = fee_quote2.convert(quote!(2000.0));

    assert_eq!(exchange.account().position().unrealized_pnl(), base!(-0.2));

    let o = Order::market(Side::Buy, size).unwrap();
    exchange.submit_order(o).unwrap();

    let _ = exchange
        .update_state(3, bba!(quote!(1999.0), quote!(2000.0)))
        .unwrap();

    assert_eq!(exchange.account().position().size(), quote!(0.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        (base!(0.8) - fee_base1 - fee_base2)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.0));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(0.8) - fee_base1 - fee_base2)
    );
}

#[test]
fn inv_long_market_win_partial() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter {
            min_quantity: quote!(0.1),
            max_quantity: quote!(0),
            step_size: quote!(0.1),
        },
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(999.0), quote!(1000.0)))
        .unwrap();

    let value = BaseCurrency::new(Dec!(0.8));
    let size = value.convert(exchange.ask());
    let o = Order::market(Side::Buy, size).unwrap();
    exchange.submit_order(o).unwrap();

    let _ = exchange
        .update_state(1, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    let fee_quote = size.fee_portion(fee_taker);
    let fee_base1: BaseCurrency = fee_quote.convert(exchange.bid());

    assert_eq!(exchange.account().position().size(), size);
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        base!(1.0) - fee_base1
    );
    assert_eq!(
        exchange.account().margin().position_margin().inner(),
        Dec!(0.8)
    );
    assert_eq!(
        exchange.account().margin().available_balance().inner(),
        Dec!(0.19952)
    );

    let _ = exchange
        .update_state(1, bba!(quote!(2000.0), quote!(2001.0)))
        .unwrap();

    let size = quote!(400.0);
    let fee_quote2 = size.fee_portion(fee_taker);
    let fee_base2: BaseCurrency = fee_quote2.convert(quote!(2000.0));

    assert_eq!(
        exchange.account().position().unrealized_pnl().inner(),
        Dec!(0.4)
    );

    let o = Order::market(Side::Sell, size).unwrap();
    exchange.submit_order(o).unwrap();

    let _ = exchange
        .update_state(2, bba!(quote!(2000.0), quote!(2001.0)))
        .unwrap();

    assert_eq!(exchange.account().position().size().inner(), Dec!(400.0));
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(
        exchange.account().position().unrealized_pnl().inner(),
        Dec!(0.2)
    );
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        base!(1.2) - fee_base1 - fee_base2
    );
    assert_eq!(
        exchange.account().margin().position_margin().inner(),
        Dec!(0.4)
    );
    assert_eq!(
        exchange.account().margin().available_balance().inner(),
        (base!(0.8) - fee_base1 - fee_base2).inner()
    );
}

#[test]
fn inv_long_market_loss_partial() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter::default(),
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(999.0), quote!(1000.0)))
        .unwrap();

    let o = Order::market(Side::Buy, quote!(800.0)).unwrap();
    exchange.submit_order(o).unwrap();
    let _ = exchange
        .update_state(1, bba!(quote!(999.0), quote!(1000.0)))
        .unwrap();

    assert_eq!(exchange.account().position().size(), quote!(800.0));

    let _ = exchange
        .update_state(1, bba!(quote!(800.0), quote!(801.0)))
        .unwrap();
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(-0.2));

    let o = Order::market(Side::Sell, quote!(400.0)).unwrap();
    exchange.submit_order(o).unwrap();
    let _ = exchange
        .update_state(1, bba!(quote!(800.0), quote!(801.0)))
        .unwrap();

    let fee_quote0 = quote!(800.0).fee_portion(fee_taker);
    let fee_base0: BaseCurrency = fee_quote0.convert(quote!(1000.0));

    let fee_quote1 = quote!(400.0).fee_portion(fee_taker);
    let fee_base1: BaseCurrency = fee_quote1.convert(quote!(800.0));

    let fee_combined: BaseCurrency = fee_base0 + fee_base1;

    assert_eq!(exchange.account().position().size(), quote!(400.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(-0.1));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        (base!(0.9) - fee_combined)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.4));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(0.5) - fee_combined)
    );
}

#[test]
fn inv_short_market_win_partial() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter::default(),
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    let o = Order::market(Side::Sell, quote!(800.0)).unwrap();
    exchange.submit_order(o).unwrap();
    let _ = exchange
        .update_state(1, bba!(quote!(999.0), quote!(1000.0)))
        .unwrap();

    assert_eq!(exchange.account().position().size(), quote!(-800.0));
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(exchange.account().margin().wallet_balance(), base!(0.99952));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(0.19952)
    );
    assert_eq!(exchange.account().margin().order_margin(), base!(0.0));
    assert_eq!(exchange.account().margin().position_margin(), base!(0.8));

    let _ = exchange
        .update_state(2, bba!(quote!(799.0), quote!(800.0)))
        .unwrap();

    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.2));

    let o = Order::market(Side::Buy, quote!(400.0)).unwrap();
    exchange.submit_order(o).unwrap();
    let _ = exchange
        .update_state(3, bba!(quote!(799.0), quote!(800.0)))
        .unwrap();

    let fee_quote0 = quote!(800.0).fee_portion(fee_taker);
    let fee_base0: BaseCurrency = fee_quote0.convert(quote!(1000.0));

    let fee_quote1 = quote!(400.0).fee_portion(fee_taker);
    let fee_base1: BaseCurrency = fee_quote1.convert(quote!(800.0));

    let fee_combined = fee_base0 + fee_base1;

    assert_eq!(exchange.account().position().size(), quote!(-400.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.1));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        (base!(1.1) - fee_combined)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.4));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(0.7) - fee_combined)
    );
}

#[test]
fn inv_short_market_loss_partial() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter {
            min_quantity: quote!(0),
            max_quantity: quote!(0),
            step_size: quote!(0.1),
        },
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(1000), quote!(1001)))
        .unwrap();

    let value = base!(0.8);
    let size: QuoteCurrency = value.convert(exchange.bid());
    let o = Order::market(Side::Sell, size).unwrap();
    exchange.submit_order(o).unwrap();

    let _ = exchange
        .update_state(1, bba!(quote!(999), quote!(1000)))
        .unwrap();

    let fee_quote1 = size.fee_portion(fee_taker);
    let fee_base1: BaseCurrency = fee_quote1.convert(quote!(1000));

    assert_eq!(exchange.account().position().size(), size.into_negative());
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        base!(1.0) - fee_base1
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.8));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(0.2) - fee_base1)
    );

    let _ = exchange
        .update_state(1, bba!(quote!(1999.0), quote!(2000.0)))
        .unwrap();

    let size = quote!(400.0);
    let fee_quote2 = size.fee_portion(fee_taker);
    let fee_base2: BaseCurrency = fee_quote2.convert(quote!(2000.0));

    assert_eq!(exchange.account().position().unrealized_pnl(), base!(-0.4));

    let o = Order::market(Side::Buy, size).unwrap();
    exchange.submit_order(o).unwrap();

    assert_eq!(exchange.account().position().size(), quote!(-400.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(-0.2));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        (base!(0.8) - fee_base1 - fee_base2)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.4));
    assert_eq!(
        exchange.account().margin().available_balance(),
        (base!(0.4) - fee_base1 - fee_base2)
    );
}

#[test]
fn inv_test_market_roundtrip() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter::default(),
        QuantityFilter::default(),
    )
    .unwrap();

    let fee_taker = config.fee_taker();
    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config);
    let _ = exchange
        .update_state(0, bba!(quote!(999.0), quote!(1000.0)))
        .unwrap();

    let value: BaseCurrency = exchange.account().margin().available_balance() * base!(0.9);
    let size = value.convert(exchange.ask());
    let buy_order = Order::market(Side::Buy, size).unwrap();
    exchange.submit_order(buy_order).unwrap();
    let _ = exchange
        .update_state(1, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    let sell_order = Order::market(Side::Sell, size).unwrap();

    exchange.submit_order(sell_order).unwrap();

    let fee_quote = size.fee_portion(fee_taker);
    let fee_base: BaseCurrency = fee_quote.convert(quote!(1000.0));

    let _ = exchange
        .update_state(2, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    assert_eq!(exchange.account().position().size(), quote!(0.0));
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert_eq!(
        exchange.account().margin().wallet_balance(),
        base!(1.0) - base!(2.0) * fee_base
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.0));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(1.0) - base!(2.0) * fee_base
    );

    let size = quote!(900.0);
    let buy_order = Order::market(Side::Buy, size).unwrap();
    exchange.submit_order(buy_order).unwrap();
    let _ = exchange
        .update_state(3, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    let size = quote!(950.0);
    let sell_order = Order::market(Side::Sell, size).unwrap();

    exchange.submit_order(sell_order).unwrap();

    let _ = exchange
        .update_state(4, bba!(quote!(998.0), quote!(1000.0)))
        .unwrap();

    assert_eq!(exchange.account().position().size(), quote!(-50.0));
    assert_eq!(exchange.account().position().entry_price(), quote!(1000.0));
    assert_eq!(exchange.account().position().unrealized_pnl(), base!(0.0));
    assert!(exchange.account().margin().wallet_balance() < base!(1.0));
    assert_eq!(exchange.account().margin().position_margin(), base!(0.05));
    assert!(exchange.account().margin().available_balance() < base!(1.0));
}

#[test]
fn inv_execute_limit() {
    if let Err(_) = pretty_env_logger::try_init() {}

    let config = Config::new(
        fee!(0.0002),
        fee!(0.0006),
        base!(1.0),
        leverage!(1.0),
        true,
        100,
        PriceFilter {
            min_price: quote!(0),
            max_price: quote!(0),
            tick_size: quote!(0.1),
            multiplier_up: Decimal::TWO,
            multiplier_down: Decimal::ZERO,
        },
        QuantityFilter::default(),
    )
    .unwrap();

    let acc_tracker = NoAccountTracker::default();
    let mut exchange = Exchange::new(acc_tracker, config.clone());
    let _ = exchange
        .update_state(0, bba!(quote!(1000.0), quote!(1001.0)))
        .unwrap();

    let o = Order::limit(Side::Buy, quote!(900.0), quote!(450.0)).unwrap();
    exchange.submit_order(o).unwrap();
    assert_eq!(exchange.account().active_limit_orders().len(), 1);
    assert_eq!(exchange.account().margin().wallet_balance(), base!(1.0));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(0.49990)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.0));
    assert_eq!(exchange.account().margin().order_margin(), base!(0.5001)); // this includes the fee too

    let (exec_orders, liq) = exchange
        .update_state(1, bba!(quote!(750.0), quote!(751.0)))
        .unwrap();
    assert!(!liq);
    assert_eq!(exec_orders.len(), 1);

    assert_eq!(exchange.bid(), quote!(750.0));
    assert_eq!(exchange.ask(), quote!(751.0));
    assert_eq!(exchange.account().active_limit_orders().len(), 0);
    assert_eq!(exchange.account().position().size(), quote!(450.0));
    assert_eq!(exchange.account().position().entry_price(), quote!(900.0));
    assert_eq!(exchange.account().margin().wallet_balance(), base!(0.9999));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(0.4999)
    );
    assert_eq!(exchange.account().margin().position_margin(), base!(0.5));
    assert_eq!(exchange.account().margin().order_margin(), base!(0.0));

    let o = Order::limit(Side::Sell, quote!(1000.0), quote!(450.0)).unwrap();
    exchange.submit_order(o).unwrap();
    assert_eq!(exchange.account().active_limit_orders().len(), 1);

    let _ = exchange
        .update_state(1, bba!(quote!(1200.0), quote!(1201.0)))
        .unwrap();

    assert_eq!(exchange.account().active_limit_orders().len(), 0);
    assert_eq!(exchange.account().position().size(), quote!(0.0));
    assert_eq!(exchange.account().margin().position_margin(), base!(0.0));
    assert_eq!(exchange.account().margin().wallet_balance(), base!(1.04981));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(1.04981)
    );

    let o = Order::limit(Side::Sell, quote!(1200.0), quote!(600.0)).unwrap();
    exchange.submit_order(o).unwrap();
    assert_eq!(exchange.account().active_limit_orders().len(), 1);

    let _ = exchange
        .update_state(2, bba!(quote!(1200.1), quote!(1200.2)))
        .unwrap();
    assert_eq!(exchange.account().position().size(), quote!(-600.0));
    assert_eq!(exchange.account().margin().position_margin(), base!(0.5));
    assert_eq!(exchange.account().margin().wallet_balance(), base!(1.04971));
    assert_eq!(
        exchange.account().margin().available_balance(),
        base!(0.54971)
    );
}
