use fpdec::Decimal;

use crate::{
    prelude::{Error, OrderError},
    quote,
    types::{Currency, MarketUpdate, Order, QuoteCurrency},
};

/// The `PriceFilter` defines the price rules for a symbol
#[derive(Debug, Clone)]
pub struct PriceFilter {
    /// Defines the minimum price allowed.
    /// Disabled if `min_price` == 0
    pub min_price: QuoteCurrency,

    /// Defines the maximum price allowed.
    /// Disabled if `max_price` == 0
    pub max_price: QuoteCurrency,

    /// Defines the intervals that a price can be increased / decreased by.
    /// For the filter to pass,
    /// (order.limit_price - min_price) % tick_size == 0
    pub tick_size: QuoteCurrency,

    /// Defines valid ranges for the order price relative to the mark price
    /// To pass this filter,
    /// order.limit_price <= mark_price * multiplier_up
    pub multiplier_up: Decimal,

    /// Defines valid ranges for the order price relative to the mark price
    /// To pass this filter,
    /// order.limit_price >= mark_price * multiplier_down
    pub multiplier_down: Decimal,
}

impl Default for PriceFilter {
    fn default() -> Self {
        Self {
            min_price: quote!(0),
            // disabled
            max_price: quote!(0),
            tick_size: quote!(1),
            multiplier_up: Decimal::TWO,
            multiplier_down: Decimal::ZERO,
        }
    }
}

impl PriceFilter {
    /// check if an `Order` is valid
    pub(crate) fn validate_order<S>(
        &self,
        order: &Order<S>,
        mark_price: QuoteCurrency,
    ) -> Result<(), OrderError>
    where
        S: Currency,
    {
        match order.limit_price() {
            Some(limit_price) => {
                if limit_price < self.min_price && self.min_price != QuoteCurrency::new_zero() {
                    return Err(OrderError::LimitPriceBelowMin);
                }
                if limit_price > self.max_price && self.max_price != QuoteCurrency::new_zero() {
                    return Err(OrderError::LimitPriceAboveMax);
                }
                if ((limit_price - self.min_price) % self.tick_size) != QuoteCurrency::new_zero() {
                    return Err(OrderError::InvalidOrderPriceStepSize);
                }
                if limit_price > mark_price * self.multiplier_up
                    && self.multiplier_up != Decimal::ZERO
                {
                    return Err(OrderError::LimitPriceAboveMultiple);
                }
                if limit_price < mark_price * self.multiplier_down
                    && self.multiplier_down != Decimal::ZERO
                {
                    return Err(OrderError::LimitPriceBelowMultiple);
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Make sure the market update conforms to the `PriceFilter` rules
    pub(crate) fn validate_market_update<S>(
        &self,
        market_update: &MarketUpdate<S>,
    ) -> Result<(), Error>
    where
        S: Currency,
    {
        match market_update {
            MarketUpdate::Bba { bid, ask } => {
                enforce_min_price(self.min_price, *bid)?;
                enforce_min_price(self.min_price, *ask)?;
                enforce_max_price(self.max_price, *bid)?;
                enforce_max_price(self.max_price, *ask)?;
                enforce_step_size(self.tick_size, *bid)?;
                enforce_step_size(self.tick_size, *ask)?;
                enforce_bid_ask_spread(*bid, *ask)?;
            }
            // We don't validate the `quantity` in the price filter, rather in the `QuantityFilter`.
            MarketUpdate::Trade { price, .. } => {
                enforce_min_price(self.min_price, *price)?;
                enforce_max_price(self.max_price, *price)?;
                enforce_step_size(self.tick_size, *price)?;
            }
            MarketUpdate::Candle {
                bid,
                ask,
                low,
                high,
            } => {
                enforce_min_price(self.min_price, *bid)?;
                enforce_min_price(self.min_price, *ask)?;
                enforce_min_price(self.min_price, *low)?;
                enforce_min_price(self.min_price, *high)?;
                enforce_max_price(self.max_price, *bid)?;
                enforce_max_price(self.max_price, *ask)?;
                enforce_max_price(self.max_price, *low)?;
                enforce_max_price(self.max_price, *high)?;
                enforce_step_size(self.tick_size, *bid)?;
                enforce_step_size(self.tick_size, *ask)?;
                enforce_step_size(self.tick_size, *low)?;
                enforce_step_size(self.tick_size, *high)?;
                enforce_bid_ask_spread(*bid, *ask)?;
                enforce_bid_ask_spread(*low, *high)?;
            }
        }
        Ok(())
    }
}

/// Errors if there is no bid-ask spread
#[inline]
fn enforce_bid_ask_spread(bid: QuoteCurrency, ask: QuoteCurrency) -> Result<(), Error> {
    if bid >= ask {
        return Err(Error::InvalidMarketUpdateBidAskSpread);
    }
    Ok(())
}

/// Make sure the price is not too low
/// Disabled if `min_price` == 0
#[inline]
fn enforce_min_price(min_price: QuoteCurrency, price: QuoteCurrency) -> Result<(), Error> {
    if price < min_price && min_price != quote!(0) {
        return Err(Error::MarketUpdatePriceTooLow);
    }
    Ok(())
}

/// Make sure the price is not too high
/// Disabled if `max_price` == 0
#[inline]
fn enforce_max_price(max_price: QuoteCurrency, price: QuoteCurrency) -> Result<(), Error> {
    if price > max_price && max_price != quote!(0) {
        return Err(Error::MarketUpdatePriceTooHigh);
    }
    Ok(())
}

/// Make sure the price conforms to the step size
#[inline]
fn enforce_step_size(step_size: QuoteCurrency, price: QuoteCurrency) -> Result<(), Error> {
    if (price % step_size) != QuoteCurrency::new_zero() {
        return Err(Error::MarketUpdatePriceStepSize);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use fpdec::Dec;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn price_filter() {
        let filter = PriceFilter {
            min_price: quote!(0.1),
            max_price: quote!(1000.0),
            tick_size: quote!(0.1),
            multiplier_up: Dec!(1.2),
            multiplier_down: Dec!(0.8),
        };
        let mark_price = quote!(100.0);

        // Market orders should always pass as the price filter only concerns limit
        // orders
        let order = Order::market(Side::Buy, base!(0.1)).unwrap();
        filter.validate_order(&order, mark_price).unwrap();
        let order = Order::market(Side::Sell, base!(0.1)).unwrap();
        filter.validate_order(&order, mark_price).unwrap();

        // Some passing orders
        let order = Order::limit(Side::Buy, quote!(99.0), base!(0.1)).unwrap();
        filter.validate_order(&order, mark_price).unwrap();
        let order = Order::limit(Side::Sell, quote!(99.0), base!(0.1)).unwrap();
        filter.validate_order(&order, mark_price).unwrap();

        // beyond max and min
        let order = Order::limit(Side::Buy, quote!(0.05), base!(0.1)).unwrap();
        assert_eq!(
            filter.validate_order(&order, mark_price),
            Err(OrderError::LimitPriceBelowMin)
        );
        let order = Order::limit(Side::Buy, quote!(1001), base!(0.1)).unwrap();
        assert_eq!(
            filter.validate_order(&order, mark_price),
            Err(OrderError::LimitPriceAboveMax)
        );

        // Test upper price band
        let order = Order::limit(Side::Buy, quote!(120), base!(0.1)).unwrap();
        filter.validate_order(&order, mark_price).unwrap();
        let order = Order::limit(Side::Buy, quote!(121), base!(0.1)).unwrap();
        assert_eq!(
            filter.validate_order(&order, mark_price),
            Err(OrderError::LimitPriceAboveMultiple)
        );

        // Test lower price band
        let order = Order::limit(Side::Buy, quote!(80), base!(0.1)).unwrap();
        filter.validate_order(&order, mark_price).unwrap();
        let order = Order::limit(Side::Buy, quote!(79), base!(0.1)).unwrap();
        assert_eq!(
            filter.validate_order(&order, mark_price),
            Err(OrderError::LimitPriceBelowMultiple)
        );

        // Test step size
        let order = Order::limit(Side::Buy, quote!(100.05), base!(0.1)).unwrap();
        assert_eq!(
            filter.validate_order(&order, mark_price),
            Err(OrderError::InvalidOrderPriceStepSize)
        );
    }
}
