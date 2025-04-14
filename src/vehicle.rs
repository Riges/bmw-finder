use core::str;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Vehicle {
    #[serde(rename = "documentId")]
    pub document_id: String,
    #[serde(rename = "vssId")]
    pub vss_id: Uuid,
    #[serde(rename = "orderingUuid")]
    pub ordering_uuid: Option<Uuid>,

    offering: Offering,
}

impl Vehicle {
    pub fn get_link(&self) -> String {
        format!(
            "https://www.bmw.fr/fr-fr/sl/stocklocator#/details/{}",
            self.vss_id
        )
    }

    pub fn get_price(&self) -> Option<f32> {
        {
            match self.offering.offer_prices {
                Some(ref offer_prices) => {
                    match offer_prices
                        .values()
                        .next()
                        .map(|price| price.offer_gross_price)
                    {
                        Some(x) => Some(x),
                        None => None,
                    }
                }
                None => None,
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Offering {
    #[serde(rename = "offerPrices")]
    offer_prices: Option<HashMap<String, OfferPrice>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct OfferPrice {
    #[serde(rename = "offerGrossPrice")]
    offer_gross_price: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{Uuid, uuid};

    #[test]
    fn test_vehicle_get_link() {
        let vehicle = Vehicle {
            document_id: "12345".to_string(),
            vss_id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            ordering_uuid: Some(Uuid::new_v4()),
            offering: Offering { offer_prices: None },
        };
        let link = vehicle.get_link();

        assert_eq!(
            link,
            "https://www.bmw.fr/fr-fr/sl/stocklocator#/details/67e55044-10b1-426f-9247-bb680e5fe0c8"
        )
    }
}
