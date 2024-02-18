use crate::{
    cornish_fisher::cornish_fisher_value_at_risk,
    types::{Currency, MarginCurrency},
    utils::decimal_to_f64,
};

/// Also called discriminant-ratio, which focuses on the added value of the
/// algorithm It uses the Cornish-Fish Value at Risk (CF-VaR)
/// It better captures the risk of the asset as it is not limited by the
/// assumption of a gaussian distribution It it time-insensitive
/// from: <https://papers.ssrn.com/sol3/papers.cfm?abstract_id=3927058>
///
/// # Parameters:
/// - `returns_account`: The ln returns of the account.
/// - `returns_bnh`: The ln returns of buy and hold aka the market returns.
/// - `wallet_balance_start`: The starting margin balance of the account.
/// - `num_trading_days`: The number of trading days.
pub fn d_ratio<M>(
    returns_account: &[f64],
    returns_bnh: &[f64],
    wallet_balance_start: M,
    num_trading_days: u64,
) -> f64
where
    M: Currency + MarginCurrency + Send,
{
    let cf_var_bnh = cornish_fisher_value_at_risk(
        returns_bnh,
        decimal_to_f64(wallet_balance_start.inner()),
        0.01,
    )
    .var;
    let cf_var_acc = cornish_fisher_value_at_risk(
        returns_account,
        decimal_to_f64(wallet_balance_start.inner()),
        0.01,
    )
    .var;

    let num_trading_days = num_trading_days as f64;

    // compute annualized returns
    let roi_acc = returns_account
        .iter()
        .fold(1.0, |acc, x| acc * x.exp())
        .powf(365.0 / num_trading_days);
    let roi_bnh = returns_bnh
        .iter()
        .fold(1.0, |acc, x| acc * x.exp())
        .powf(365.0 / num_trading_days);

    let rtv_acc = roi_acc / cf_var_acc;
    let rtv_bnh = roi_bnh / cf_var_bnh;
    debug!(
            "roi_acc: {:.2}, roi_bnh: {:.2}, cf_var_bnh: {:.8}, cf_var_acc: {:.8}, rtv_acc: {}, rtv_bnh: {}",
            roi_acc, roi_bnh, cf_var_bnh, cf_var_acc, rtv_acc, rtv_bnh,
        );

    (1.0 + (roi_acc - roi_bnh) / roi_bnh.abs()) * (cf_var_bnh / cf_var_acc)
}
