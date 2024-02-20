use crate::{
    account_tracker::statistical_moments,
    types::{Currency, LnReturns},
    Result,
};

/// Contains the cornish fisher outputs.
pub struct CornishFisherOutput<M> {
    pub exp: f64,
    pub var: f64,
    pub asset_value_at_risk: M,
}

/// Compute the Cornish-Fisher Value at Risk (CF-VaR)
///
/// # Arguments:
/// - 'log_returns': natural logarithmic return series: (p1 / p0).ln()
/// - `asset_value`: Serves as the base from which the `asset_value_at_risk` is computed.
/// - `confidence_interval`: in range [0.0, 1.0], usually something like 0.01 or
/// 0.05.
///
/// # Returns:
/// tuple containing (cf_exp, cf_var, cf_asset_value)
/// of most importance is cf_var which if the actual CF-VaR
pub(crate) fn cornish_fisher_value_at_risk<'a, C>(
    log_returns: &LnReturns<'a, f64>,
    asset_value: C,
    confidence_interval: f64,
) -> Result<CornishFisherOutput<C>>
where
    C: Currency,
{
    let stats = statistical_moments(log_returns.0);

    let quantile = distrs::Normal::ppf(confidence_interval, 0.0, 1.0);

    let exp = quantile
        + (quantile.powi(2) - 1.0) * stats.skew / 6.0
        + (quantile.powi(3) - 3.0 * quantile) * stats.excess_kurtosis / 24.0
        - (2.0 * quantile.powi(3) - 5.0 * quantile) * stats.skew.powi(2) / 36.0;

    let var = stats.mean + stats.std_dev * exp;

    // If these were percent returns we'd use the commented out one.
    // But her we use ln returns, so we take the latter one.
    //let asset_value_at_risk = asset_value * (1.0 + cf_var);
    let asset_value_at_risk = asset_value - (asset_value * C::new(var.exp().try_into()?));

    Ok(CornishFisherOutput {
        exp,
        var,
        asset_value_at_risk,
    })
}
