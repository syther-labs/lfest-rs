use crate::account_tracker::statistical_moments;

/// Contains the cornish fisher outputs.
pub struct CornishFisherOutput {
    pub exp: f64,
    pub var: f64,
    pub asset_value: f64,
}

/// Compute the Cornish-Fisher Value at Risk (CF-VaR)
///
/// # Arguments:
/// - 'log_returns': logarithmic return series: (p1 / p0).ln()
/// - `asset_value`: current asset value
/// - `confidence_interval`: in range [0.0, 1.0], usually something like 0.01 or
/// 0.05.
///
/// # Returns:
/// tuple containing (cf_exp, cf_var, cf_asset_value)
/// of most importance is cf_var which if the actual CF-VaR
pub(crate) fn cornish_fisher_value_at_risk(
    log_returns: &[f64],
    asset_value: f64,
    confidence_interval: f64,
) -> CornishFisherOutput {
    let stats = statistical_moments(log_returns);

    let quantile = distrs::Normal::ppf(confidence_interval, 0.0, 1.0);
    let exp = quantile
        + (quantile.powi(2) - 1.0) * stats.skew / 6.0
        + (quantile.powi(3) - 3.0 * quantile) * stats.kurtosis / 24.0
        - (2.0 * quantile.powi(3) - 5.0 * quantile) * stats.skew.powi(2) / 36.0;
    let var = stats.mean + stats.std_dev * exp;
    //let cf_asset_value = asset_value * (1.0 + cf_var); // like in the paper, but
    // wrong as the underlying returns are logarithmic
    let asset_value = asset_value - (asset_value * var.exp());

    CornishFisherOutput {
        exp,
        var,
        asset_value,
    }
}
