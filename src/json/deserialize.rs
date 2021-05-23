use chrono::{DateTime, NaiveDateTime, Utc};
struct DateTimeFromCustomFormatVisitor;

pub fn deserialize_datetime<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(DateTimeFromCustomFormatVisitor)
}

impl<'de> de::Visitor<'de> for DateTimeFromCustomFormatVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a datetime string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.fZ") {
            Ok(ndt) => Ok(DateTime::from_utc(ndt, Utc)),
            Err(e) => Err(E::custom(format!("Parse error {} for {}", e, value))),
        }
    }
}