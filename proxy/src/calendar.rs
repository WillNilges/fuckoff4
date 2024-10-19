use anyhow::anyhow;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Deserialize;
use std::env;
use url::form_urlencoded;

// Struct that fits the dateTime field of the Google Calendar API
// response
#[derive(Debug, Deserialize, Clone)]
pub struct EventTimeInfo {
    #[serde(rename = "dateTime")]
    pub date_time: Option<DateTime<Utc>>, // All-day events only have a date
    pub date: Option<NaiveDate>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

// Struct that fits a single event from the Google Calendar
// API response
#[derive(Debug, Deserialize, Clone)]
pub struct Event {
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: EventTimeInfo,
    pub end: EventTimeInfo,
}

impl Event {
    pub fn format_2004(&self) -> String {
        if let Some(start_time) = &self.start.date_time {
            let duration_until = Self::time_until(start_time);
            if duration_until > Duration::zero() {
                let t = Self::format_duration(duration_until);
                return format!("{}\nIn {}", self.summary, t);
            } else {
                // If that didn't work, then the event is probably already going.
                // Check if we can get the time until.
                if let Some(end_time) = &self.end.date_time {
                    let duration_until = Self::time_until(end_time);
                    if duration_until > Duration::zero() {
                        let t = Self::format_duration(duration_until);
                        return format!("{}\n{} Left", self.summary, t);
                    }
                }
            }
        }

        // If we don't have any datetime info, then
        // just return the title of the event
        self.summary.clone()
    }

    fn time_until(timestamp: &DateTime<Utc>) -> Duration {
        // let parsed_timestamp = DateTime::parse_from_rfc3339(timestamp)
        //     .expect("Failed to parse timestamp")
        //     .with_timezone(&Utc);

        let current_time = Utc::now();

        timestamp.signed_duration_since(current_time)
    }

    fn format_duration(duration: Duration) -> String {
        let seconds = duration.num_seconds();
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let remaining_seconds = seconds % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, remaining_seconds)
    }
}

// Object used to grok payload returned directly by the Google Calendar
// API
#[derive(Clone, Debug, Deserialize)]
pub struct CalendarEvents {
    pub kind: String,
    pub items: Vec<Event>,
}

impl CalendarEvents {
    // Call the Google Calendar API and return a usable object from that
    pub async fn new() -> anyhow::Result<Self> {
        let gcal_resp = Self::query_gcal().await?;
        let events: Self = serde_json::from_str::<CalendarEvents>(gcal_resp.as_str())
            .map_err(|e| anyhow!("{}", e))?;
        Ok(events)
    }

    // Update the contents of this struct
    pub async fn update(&mut self) -> anyhow::Result<()> {
        let gcal_resp = Self::query_gcal().await?;
        self.items = serde_json::from_str::<CalendarEvents>(gcal_resp.as_str())
            .map_err(|e| anyhow!("{}", e))?
            .items;
        Ok(())
    }

    // Perform Google Calendar API Call
    async fn query_gcal() -> anyhow::Result<String> {
        let utc: DateTime<Utc> = Utc::now();
        let iso_time = utc.to_rfc3339();
        let api_key = env::var("API_KEY")?;
        let calendar_id = env::var("CALENDAR_ID")?;

        let params = [
            ("maxResults", "10"),
            ("orderBy", "startTime"),
            ("showDeleted", "false"),
            ("singleEvents", "true"),
            ("timeMin", &iso_time),
            (
                "fields",
                "kind,items(location, start, end, summary, description)",
            ),
            ("key", &api_key),
        ];

        // Encode parameters into a query string
        let encoded_params: String = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params.iter())
            .finish();

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events?{}",
            calendar_id, encoded_params
        );

        let body = reqwest::get(url).await?.text().await?;

        Ok(body)
    }

    pub fn get_next_at_location(&self, location: &str) -> Option<Event> {
        self.items
            .iter()
            .find(|e| {
                e.location.as_ref().is_some_and(|l| l.contains(location))
                    && !e.summary.contains("CANCEL")
            })
            .cloned()
    }

    // Check if the provided range of DateTimes overlap with anything
    // we already know about.
    pub fn is_free_at_location(
        &self,
        location: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> bool {
        let query = (start, end);
        for e in &self.items {
            if let Some(ref l) = e.location {
                if l.contains(location) {
                    let e_times = (e.start.date_time.unwrap(), e.end.date_time.unwrap());
                    if Self::is_overlap(&query, &e_times) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn is_overlap(
        proposed: &(DateTime<Utc>, DateTime<Utc>),
        existing: &(DateTime<Utc>, DateTime<Utc>),
    ) -> bool {
        proposed.0 < existing.1 && proposed.1 > existing.0
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar::{CalendarEvents, Event, EventTimeInfo};
    use chrono::prelude::*;

    #[test]
    fn test_get_next_at_location() {
        let events = CalendarEvents {
            kind: "".to_string(),
            items: vec![
                Event {
                    summary: "Test".to_string(),
                    description: None,
                    location: Some("Lounge".to_string()),
                    start: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                    end: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 1, 0, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                },
                Event {
                    summary: "Test Number 2".to_string(),
                    description: None,
                    location: Some("Lounge".to_string()),
                    start: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 2, 30, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                    end: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 4, 0, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                },
            ],
        };

        // The next event should be "Test"
        assert_eq!(
            events.get_next_at_location("Lounge").unwrap().summary,
            "Test"
        );
    }

    #[test]
    fn test_is_free_at_location() {
        let events = CalendarEvents {
            kind: "".to_string(),
            items: vec![
                Event {
                    summary: "Test".to_string(),
                    description: None,
                    location: Some("Lounge".to_string()),
                    start: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                    end: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 1, 0, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                },
                Event {
                    summary: "Test Number 2".to_string(),
                    description: None,
                    location: Some("Lounge".to_string()),
                    start: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 2, 30, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                    end: EventTimeInfo {
                        date_time: Some(Utc.with_ymd_and_hms(2020, 1, 1, 4, 0, 0).unwrap()),
                        date: None,
                        time_zone: None,
                    },
                },
            ],
        };

        // The room should be taken
        let mut query_start = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let mut query_end = Utc.with_ymd_and_hms(2020, 1, 1, 1, 30, 0).unwrap();
        assert!(!events.is_free_at_location("Lounge", query_start, query_end));

        // The room should be free
        query_start = Utc.with_ymd_and_hms(2020, 2, 1, 0, 0, 0).unwrap();
        query_end = Utc.with_ymd_and_hms(2020, 2, 1, 1, 30, 0).unwrap();
        assert!(events.is_free_at_location("Lounge", query_start, query_end));
    }
}
