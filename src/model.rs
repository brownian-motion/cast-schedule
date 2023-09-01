use chrono::offset::*;
use chrono::prelude::*;
use chrono::Duration;
use std::cmp::{max, min};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DefiniteTimeRange<TZ: TimeZone> {
    pub start: DateTime<TZ>,
    pub end: DateTime<TZ>,
}

impl<TZ> DefiniteTimeRange<TZ>
where
    TZ: TimeZone,
{
    pub fn to_indefinite(&self) -> IndefiniteTimeRange<TZ> {
        IndefiniteTimeRange {
            start: Some(self.start.clone()),
            end: Some(self.end.clone()),
        }
    }

    pub fn with_timezone<TZ2: TimeZone>(&self, timezone: &TZ2) -> DefiniteTimeRange<TZ2> {
        DefiniteTimeRange {
            start: self.start.with_timezone(timezone),
            end: self.end.with_timezone(timezone),
        }
    }

    pub fn duration(&self) -> Duration {
        self.end.clone() - self.start.clone()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IndefiniteTimeRange<TZ: TimeZone> {
    pub start: Option<DateTime<TZ>>,
    pub end: Option<DateTime<TZ>>,
}

impl<TZ> IndefiniteTimeRange<TZ>
where
    TZ: TimeZone,
{
    pub fn clamp_to(self, bounds: &DefiniteTimeRange<TZ>) -> DefiniteTimeRange<TZ> {
        DefiniteTimeRange {
            start: match self.start {
                Some(start) => max(start, bounds.start.clone()),
                None => bounds.start.clone(),
            },
            end: match self.end {
                Some(end) => min(end, bounds.end.clone()),
                None => bounds.end.clone(),
            },
        }
    }

    pub fn with_timezone<TZ2: TimeZone>(&self, timezone: &TZ2) -> IndefiniteTimeRange<TZ2> {
        IndefiniteTimeRange {
            start: self.start.clone().map(|t| t.with_timezone(timezone)),
            end: self.end.clone().map(|t| t.with_timezone(timezone)),
        }
    }
}

#[derive(Debug)]
pub struct CalendarEvent {
    pub name: String,
    pub times: IndefiniteTimeRange<Local>,
}

// impl CalendarEvent {
//     pub fn duration(&self) -> Duration {
//         self.times.end.signed_duration_since(self.times.start)
//     }
// }

#[derive(Debug, Eq, PartialEq)]
pub struct CurrentStatus {
    pub has_meeting: bool,
    pub mic_active: bool,
    pub in_meeting: bool,
}

#[derive(Debug)]
pub struct Model {
    pub events: Vec<CalendarEvent>,
    pub status: CurrentStatus,
}
