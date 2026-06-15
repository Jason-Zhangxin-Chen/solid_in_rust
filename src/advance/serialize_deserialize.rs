use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;

/// A date stored internally as YYYY-MM-DD, but serialized as "MM/DD/YYYY"
#[derive(Debug, Clone, PartialEq)]
struct UsDate {
    year: u16,
    month: u8,
    day: u8,
}

impl UsDate {
    fn new(year: u16, month: u8, day: u8) -> Result<Self, String> {
        if month == 0 || month > 12 || day == 0 || day > 31 {
            return Err("Invalid date".to_string());
        }
        Ok(Self { year, month, day })
    }
}

// ─── Custom Serialize ─────────────────────────────────────────────
impl Serialize for UsDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as "MM/DD/YYYY" string
        let formatted = format!("{:02}/{:02}/{}", self.month, self.day, self.year);
        serializer.serialize_str(&formatted)
    }
}

// ─── Custom Deserialize ─────────────────────────────────────────────
impl<'de> Deserialize<'de> for UsDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UsDateVisitor;

        impl<'de> Visitor<'de> for UsDateVisitor {
            type Value = UsDate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a date string in MM/DD/YYYY format")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let parts: Vec<&str> = value.split('/').collect();
                if parts.len() != 3 {
                    return Err(E::custom(format!("expected MM/DD/YYYY, got {}", value)));
                }

                let month = parts[0].parse::<u8>()
                    .map_err(|e| E::custom(format!("invalid month: {}", e)))?;
                let day = parts[1].parse::<u8>()
                    .map_err(|e| E::custom(format!("invalid day: {}", e)))?;
                let year = parts[2].parse::<u16>()
                    .map_err(|e| E::custom(format!("invalid year: {}", e)))?;

                UsDate::new(year, month, day)
                    .map_err(|e| E::custom(e))
            }
        }

        deserializer.deserialize_str(UsDateVisitor)
    }
}

// ─── Using it in a struct ─────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
struct Event {
    name: String,
    #[serde(with = "UsDate")]  // You can also use this field attribute
    date: UsDate,
}

fn main() {
    let event = Event {
        name: "Conference".to_string(),
        date: UsDate::new(2026, 6, 15).unwrap(),
    };

    // Serialize
    let json = serde_json::to_string_pretty(&event).unwrap();
    println!("Serialized:\n{}\n", json);
    // {
    //   "name": "Conference",
    //   "date": "06/15/2026"
    // }

    // Deserialize
    let input = r#"{"name":"Meeting","date":"12/25/2026"}"#;
    let parsed: Event = serde_json::from_str(input).unwrap();
    println!("Deserialized: {:?}", parsed);
    // Deserialized: Event { name: "Meeting", date: UsDate { year: 2026, month: 12, day: 25 } }
}