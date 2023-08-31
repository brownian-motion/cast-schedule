
use google_calendar::StatusCode;
use google_calendar::types::Event;
use super::calendar::*;
use super::errors::*;
use crate::model::*;
use google_calendar::types::OrderBy;
use chrono::prelude::*;

pub struct GoogleCalendar {
    client: google_calendar::Client,
}

impl Calendar for GoogleCalendar {
    async fn get_events_on(&self, date: DefiniteTimeRange<Local>) -> Result<Vec<CalendarEvent>, CalendarError> {
        // .list() has a lot of optional values, which are not put in the request if the "empty"/default value
        let empty_array = vec![];
        let time_min = date.start.to_rfc3339();
        let time_max = date.end.to_rfc3339();
        let events = self.client.events().list(
            "primary", // which calendar
            "",        // event ID
            0,         // attendees
            50,        // max results
            OrderBy::StartTime,
            "",           // page token
            &empty_array, // private_extended_property
            "",           // search query
            &empty_array, // shared_extended_property
            false,        // show deleted
            false,        // show hidden invitations
            true,         // expand repeating meetings into single events
            &time_min,
            &time_max,
            "", // use the calendar's time zone
            "", // filter by modification time
        ).await;

        match events {
            Ok(events) => parse_events(events),
            Err(e) => {
                eprintln!("could not fetch from google calendar: {:?}", e);
                Err(CalendarError::FetchError)
            }
        }
    }
}

fn parse_events(response: google_calendar::Response<Vec<Event>>) -> Result<Vec<CalendarEvent>, CalendarError> {
    if response.status != StatusCode::OK {
        eprintln!("HTTP error {:?} fetching events", response.status);
        return Err(CalendarError::FetchError);
    }

    Ok(response.body.into_iter().map(|g_event| CalendarEvent{
        name: g_event.etag.to_string(),
        times: IndefiniteTimeRange{
            start: g_event.start.and_then(|t| t.date_time).map(|t| t.with_timezone(&Local)),
            end: g_event.end.and_then(|t| t.date_time).map(|t| t.with_timezone(&Local)),
        }
    }).collect())
}