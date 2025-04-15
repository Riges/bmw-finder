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

    #[serde(rename = "offering")]
    offering: Offering,

    #[serde(rename = "vehicleSpecification")]
    vehicle_specification: VehicleSpecification,

    #[serde(rename = "price")]
    price: VehiclePrice,
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

    pub fn get_discount_percentage(&self) -> Option<f32> {
        let default_price = self.price.vehicle_net_price;
        let offer_price = self.get_price()?;
        Some((default_price - offer_price) / default_price * 100.0)
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct VehicleSpecification {
    #[serde(rename = "modelAndOption")]
    model_and_option: ModelAndOption,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ModelAndOption {
    #[serde(rename = "equipments")]
    equipments: HashMap<String, Equipment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Equipment {
    #[serde(rename = "name")]
    name: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct VehiclePrice {
    #[serde(rename = "equipmentsTotalPrice")]
    equipments_total_price: f32,

    #[serde(rename = "vehicleNetPrice")]
    vehicle_net_price: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{Uuid, uuid};

    #[test]
    fn get_link() {
        let vehicle = Vehicle {
            document_id: "12345".to_string(),
            vss_id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            ordering_uuid: Some(Uuid::new_v4()),
            offering: Offering { offer_prices: None },
            price: VehiclePrice {
                equipments_total_price: 0.0,
                vehicle_net_price: 0.0,
            },
            vehicle_specification: VehicleSpecification {
                model_and_option: ModelAndOption {
                    equipments: HashMap::new(),
                },
            },
        };
        let link = vehicle.get_link();

        assert_eq!(
            link,
            "https://www.bmw.fr/fr-fr/sl/stocklocator#/details/67e55044-10b1-426f-9247-bb680e5fe0c8"
        )
    }

    mod get_price {
        use super::*;
        use uuid::Uuid;

        #[test]
        fn should_return_price() {
            let vehicle = Vehicle {
                document_id: "12345".to_string(),
                vss_id: Uuid::new_v4(),
                ordering_uuid: Some(Uuid::new_v4()),
                offering: Offering {
                    offer_prices: Some(HashMap::from([(
                        "FR".to_string(),
                        OfferPrice {
                            offer_gross_price: 100.0,
                        },
                    )])),
                },
                price: VehiclePrice {
                    equipments_total_price: 0.0,
                    vehicle_net_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
            };

            assert_eq!(vehicle.get_price(), Some(100.0));
        }

        #[test]
        fn should_return_none_when_no_offers_exist() {
            let vehicle = Vehicle {
                document_id: "12345".to_string(),
                vss_id: Uuid::new_v4(),
                ordering_uuid: Some(Uuid::new_v4()),
                offering: Offering { offer_prices: None },
                price: VehiclePrice {
                    equipments_total_price: 0.0,
                    vehicle_net_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
            };

            assert_eq!(vehicle.get_price(), None);
        }
    }

    mod get_discount_percentage {
        use super::*;
        use uuid::Uuid;

        #[test]
        fn should_return_discount_percentage() {
            let vehicle = Vehicle {
                document_id: "12345".to_string(),
                vss_id: Uuid::new_v4(),
                ordering_uuid: Some(Uuid::new_v4()),
                offering: Offering {
                    offer_prices: Some(HashMap::from([(
                        "FR".to_string(),
                        OfferPrice {
                            offer_gross_price: 75.0,
                        },
                    )])),
                },
                price: VehiclePrice {
                    equipments_total_price: 0.0,
                    vehicle_net_price: 100.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
            };

            assert_eq!(vehicle.get_discount_percentage(), Some(25.0));
        }

        #[test]
        fn should_return_none_if_price_doesnt_exist() {
            let vehicle = Vehicle {
                document_id: "12345".to_string(),
                vss_id: Uuid::new_v4(),
                ordering_uuid: Some(Uuid::new_v4()),
                offering: Offering { offer_prices: None },
                price: VehiclePrice {
                    equipments_total_price: 0.0,
                    vehicle_net_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
            };

            assert_eq!(vehicle.get_discount_percentage(), None);
        }
    }
}
