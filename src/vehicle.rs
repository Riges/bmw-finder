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

    #[serde(rename = "ordering")]
    ordering: Ordering,
}

impl Vehicle {
    pub fn get_link(&self) -> String {
        format!(
            "https://www.bmw.fr/fr-fr/sl/{}#/details/{}",
            match self.ordering.order_data.usage_state.as_str() {
                "NEW" => "stocklocator",
                _ => "stocklocator_uc",
            },
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
        let default_price = self.price.vehicle_gross_price;
        let offer_price = self.get_price()?;
        Some((default_price - offer_price) / default_price * 100.0)
    }

    pub fn has_equipment_name_like(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let name = &name.to_lowercase();

        self.vehicle_specification
            .model_and_option
            .equipments
            .iter()
            .any(|(_, equipment)| {
                equipment
                    .name
                    .iter()
                    .any(|(_, value)| value.to_lowercase().contains(name))
            })
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
    #[serde(rename = "vehicleGrossPrice")]
    vehicle_gross_price: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Ordering {
    #[serde(rename = "orderData")]
    order_data: OrderData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct OrderData {
    #[serde(rename = "usageState")]
    usage_state: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{Uuid, uuid};

    #[test]
    fn get_new_link_when_usage_state_is_new() {
        let vehicle = Vehicle {
            document_id: "12345".to_string(),
            vss_id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            ordering_uuid: Some(Uuid::new_v4()),
            offering: Offering { offer_prices: None },
            price: VehiclePrice {
                vehicle_gross_price: 0.0,
            },
            vehicle_specification: VehicleSpecification {
                model_and_option: ModelAndOption {
                    equipments: HashMap::new(),
                },
            },
            ordering: Ordering {
                order_data: OrderData {
                    usage_state: "NEW".to_string(),
                },
            },
        };
        let link = vehicle.get_link();

        assert_eq!(
            link,
            "https://www.bmw.fr/fr-fr/sl/stocklocator#/details/67e55044-10b1-426f-9247-bb680e5fe0c8"
        )
    }

    #[test]
    fn get_used_link_when_usage_state_is_used() {
        let vehicle = Vehicle {
            document_id: "12345".to_string(),
            vss_id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            ordering_uuid: Some(Uuid::new_v4()),
            offering: Offering { offer_prices: None },
            price: VehiclePrice {
                vehicle_gross_price: 0.0,
            },
            vehicle_specification: VehicleSpecification {
                model_and_option: ModelAndOption {
                    equipments: HashMap::new(),
                },
            },
            ordering: Ordering {
                order_data: OrderData {
                    usage_state: "USED".to_string(),
                },
            },
        };
        let link = vehicle.get_link();

        assert_eq!(
            link,
            "https://www.bmw.fr/fr-fr/sl/stocklocator_uc#/details/67e55044-10b1-426f-9247-bb680e5fe0c8"
        )
    }

    #[test]
    fn get_used_link_when_state_is_not_new() {
        let vehicle = Vehicle {
            document_id: "12345".to_string(),
            vss_id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            ordering_uuid: Some(Uuid::new_v4()),
            offering: Offering { offer_prices: None },
            price: VehiclePrice {
                vehicle_gross_price: 0.0,
            },
            vehicle_specification: VehicleSpecification {
                model_and_option: ModelAndOption {
                    equipments: HashMap::new(),
                },
            },
            ordering: Ordering {
                order_data: OrderData {
                    usage_state: "DEALER_YOUNG_USED".to_string(),
                },
            },
        };
        let link = vehicle.get_link();

        assert_eq!(
            link,
            "https://www.bmw.fr/fr-fr/sl/stocklocator_uc#/details/67e55044-10b1-426f-9247-bb680e5fe0c8"
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
                    vehicle_gross_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
                ordering: Ordering {
                    order_data: OrderData {
                        usage_state: "NEW".to_string(),
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
                    vehicle_gross_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
                ordering: Ordering {
                    order_data: OrderData {
                        usage_state: "NEW".to_string(),
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
                    vehicle_gross_price: 100.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
                ordering: Ordering {
                    order_data: OrderData {
                        usage_state: "NEW".to_string(),
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
                    vehicle_gross_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
                ordering: Ordering {
                    order_data: OrderData {
                        usage_state: "NEW".to_string(),
                    },
                },
            };

            assert_eq!(vehicle.get_discount_percentage(), None);
        }
    }

    mod has_equipment_name_like {
        use super::*;
        use uuid::Uuid;

        #[test]
        fn should_return_true_if_name_exists() {
            let vehicle = Vehicle {
                document_id: "12345".to_string(),
                vss_id: Uuid::new_v4(),
                ordering_uuid: Some(Uuid::new_v4()),
                offering: Offering { offer_prices: None },
                price: VehiclePrice {
                    vehicle_gross_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::from([(
                            "TEST42".to_string(),
                            Equipment {
                                name: HashMap::from([
                                    ("default_FR".to_string(), "Test asdasdasd".to_string()),
                                    ("fr_FR".to_string(), "Another name".to_string()),
                                ]),
                            },
                        )]),
                    },
                },
                ordering: Ordering {
                    order_data: OrderData {
                        usage_state: "NEW".to_string(),
                    },
                },
            };

            let result = vehicle.has_equipment_name_like("Test");

            assert_eq!(result, true);
        }

        #[test]
        fn should_return_false_if_name_doesnt_exist() {
            let vehicle = Vehicle {
                document_id: "12345".to_string(),
                vss_id: Uuid::new_v4(),
                ordering_uuid: Some(Uuid::new_v4()),
                offering: Offering { offer_prices: None },
                price: VehiclePrice {
                    vehicle_gross_price: 0.0,
                },
                vehicle_specification: VehicleSpecification {
                    model_and_option: ModelAndOption {
                        equipments: HashMap::new(),
                    },
                },
                ordering: Ordering {
                    order_data: OrderData {
                        usage_state: "NEW".to_string(),
                    },
                },
            };

            assert_eq!(vehicle.has_equipment_name_like("Test"), false);
        }
    }
}
