
use chrono::Local;
use super::errors::*;
use crate::model::*;

pub trait Calendar {
    async fn get_events_on(&self, when: DefiniteTimeRange<Local>) -> Result<Vec<CalendarEvent>, CalendarError>;
}

pub trait CalendarProvider {
    type Calendar: Calendar;
    fn login(&mut self) -> Result<Self::Calendar, CalendarError>;
}
