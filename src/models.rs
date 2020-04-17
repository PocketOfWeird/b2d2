use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Order {
    #[serde(rename = "Order #")]
    pub id: u32,
    #[serde(rename = "Fulfillment Type")]
    pub fulfillment: Option<String>,
    #[serde(rename = "Paid")]
    pub paid: String,
    #[serde(rename = "Item")]
    pub item: String,
    #[serde(rename = "Unit Size")]
    pub unit_size: Option<String>,
    #[serde(rename = "Quantity")]
    pub quantity: Option<u32>,
    #[serde(rename = "Price")]
    pub price: Option<String>,
    #[serde(rename = "Customer")]
    pub customer: String,
    #[serde(rename = "Pickup Address")]
    pub addr_pickup: Option<String>,
    #[serde(rename = "Delivery Address")]
    pub addr_delivery: Option<String>,
}