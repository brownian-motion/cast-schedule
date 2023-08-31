use chrono::*;

use std::ops::{Add, Div, Mul, Sub};

fn lerp<T>(num: T, source_start: T, source_end: T, dest_start: T, dest_end: T) -> T
where
    T: Sized + Copy + Add<Output = T> + Div<Output = T> + Mul<Output = T> + Sub<Output = T>,
{
    (num - source_start) * (dest_end - dest_start) / (source_end - source_start) + dest_start
}

struct TimeRangeDrawer<TZ: TimeZone> {
    start: DateTime<TZ>,
    duration: Duration,
    style: Style,
}

impl<TZ> TimeRangeDrawer<TZ>
where
    TZ: TimeZone,
{
    fn end(&self) -> DateTime<TZ> {
        self.start.clone() + self.duration
    }

    fn timezone(&self) -> TZ {
        self.start.timezone()
    }
}

impl<TZ> Drawer for TimeRangeDrawer<TZ>
where
    TZ: TimeZone,
{
    type Subject = CalendarEvent;

    fn draw(&self, event: &CalendarEvent, bounds: &DrawingBounds) -> Vec<Drawing> {
        if event.times.start.is_some_and(|start| start >= self.end()) {
            return vec![];
        }

        if event.times.end.is_some_and(|end| end <= self.start) {
            return vec![];
        }

        let event_bounds =
            event
                .times
                .with_timezone(&self.start.timezone())
                .clamp_to(&DefiniteTimeRange {
                    start: self.start.clone(),
                    end: self.end(),
                });

        let DefiniteTimeRange {
            start: event_start,
            end: event_end,
        } = event_bounds;
        let event_duration = event_end - event_start.clone();

        let event_height = lerp::<u32>(
            event_duration.num_minutes() as u32,
            0,
            self.duration.num_minutes() as u32,
            0,
            bounds.height,
        );
        let event_top = lerp::<u32>(
            (event_start - self.start.clone()).num_minutes() as u32,
            0,
            self.duration.num_minutes() as u32,
            bounds.top,
            bounds.top + bounds.height,
        );

        vec![Drawing::new()
            .with_shape(Shape::Rectangle {
                width: bounds.width,
                height: event_height,
            })
            .with_xy(bounds.left as f32, event_top as f32)
            .with_style(self.style.clone())]
    }
}

pub struct CalendarDrawer<TZ> {
    start_date: NaiveDate,
    num_days: u32,
    day_start_time: NaiveTime,
    day_duration: Duration,
    time_zone: TZ,
    base_style: Style,
}

impl<TZ: TimeZone> CalendarDrawer<TZ> {
    fn single_day_drawer(&self, day_num: u32) -> TimeRangeDrawer<TZ> {
        TimeRangeDrawer {
            start: self.start_of_day(day_num),
            duration: self.day_duration,
            style: self.base_style.clone(), // TODO: different colors for past/future
        }
    }

    fn day_end_time(&self) -> NaiveTime {
        self.day_start_time + self.day_duration
    }

    fn date(&self, day_num: u32) -> NaiveDate {
        self.start_date + Days::new(day_num as u64)
    }

    fn start_of_day(&self, day_num: u32) -> DateTime<TZ> {
        self.date(day_num)
            .and_time(self.day_start_time)
            .and_local_timezone(self.time_zone.clone())
            .earliest()
            .expect("unrepresentable datetime")
    }

    fn end_of_day(&self, day_num: u32) -> DateTime<TZ> {
        self.date(day_num)
            .and_time(self.day_end_time())
            .and_local_timezone(self.time_zone.clone())
            .earliest()
            .expect("unrepresentable datetime")
    }

    fn start_of_first_day(&self) -> DateTime<TZ> {
        self.start_of_day(0)
    }

    fn end_of_last_day(&self) -> DateTime<TZ> {
        self.end_of_day(self.num_days - 1)
    }
}

use super::*;
use crate::model::*;
use draw::*;

impl<TZ> Drawer for CalendarDrawer<TZ>
where
    TZ: TimeZone,
{
    type Subject = [CalendarEvent];

    fn draw(&self, events: &Self::Subject, bounds: &DrawingBounds) -> Vec<Drawing> {
        (0..self.num_days)
            .into_iter()
            .flat_map(|day_num| {
                let drawer = self.single_day_drawer(day_num);
                let sub_bounds = DrawingBounds {
                    left: bounds.left + day_num * bounds.width / self.num_days,
                    top: bounds.top,
                    width: bounds.width / self.num_days,
                    height: bounds.height,
                };
                events
                    .iter()
                    .flat_map(move |event| drawer.draw(event, &sub_bounds).into_iter())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DAY: NaiveDate = NaiveDate::from_ymd_opt(2022, 9, 1).unwrap();
    const TEST_START_TIME: NaiveTime = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    const TEST_END_TIME: NaiveTime = NaiveTime::from_hms_opt(18, 0, 0).unwrap(); // 10 hrs for easy subdivision

    const TEST_BOUNDS: DrawingBounds = DrawingBounds {
        top: 0,
        left: 0,
        width: 200,
        height: 100,
    };

    fn test_drawer() -> CalendarDrawer<Local> {
        CalendarDrawer {
            start_date: TEST_DAY,
            num_days: 2,
            day_start_time: TEST_START_TIME,
            day_duration: TEST_END_TIME - TEST_START_TIME,
            time_zone: Local,
            base_style: Style::default(),
        }
    }

    #[test]
    fn given_no_events_then_nothing_is_displayed() {
        let events = vec![];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert!(drawings.is_empty());
    }

    #[test]
    fn given_infinite_event_then_both_days_are_filled() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: None,
                end: None,
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(2, drawings.len());
        assert_eq!(draw::Point { x: 0.0, y: 0.0 }, drawings[0].position);
        assert!(matches!(
            drawings[0].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 100
            })
        ));
        assert_eq!(draw::Point { x: 100.0, y: 0.0 }, drawings[1].position);
        assert!(matches!(
            drawings[1].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 100
            })
        ));
    }

    #[test]
    fn given_single_midday_event_then_single_rectangle() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 1, 10, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 1, 12, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(1, drawings.len());
        assert_eq!(draw::Point { x: 0.0, y: 20.0 }, drawings[0].position);
        assert!(matches!(
            drawings[0].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 20
            })
        ));
    }

    #[test]
    fn given_single_event_when_event_starts_early_then_single_clamped_rectangle() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 1, 0, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 1, 12, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(1, drawings.len());
        assert_eq!(draw::Point { x: 0.0, y: 0.0 }, drawings[0].position);
        assert!(matches!(
            drawings[0].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 40
            })
        ));
    }

    #[test]
    fn given_single_event_when_event_runs_late_then_single_clamped_rectangle() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 1, 14, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 1, 22, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(1, drawings.len());
        assert_eq!(draw::Point { x: 0.0, y: 60.0 }, drawings[0].position);
        assert!(matches!(
            drawings[0].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 40
            })
        ));
    }

    #[test]
    fn given_single_event_when_event_is_day_before_then_draw_nothing() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 8, 31, 14, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 8, 31, 22, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(0, drawings.len());
    }

    #[test]
    fn given_single_event_when_event_is_day_after_then_draw_nothing() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 3, 14, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 3, 22, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(0, drawings.len());
    }

    #[test]
    fn given_single_event_when_event_ends_at_start_of_first_day_then_draw_nothing() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 1, 6, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 1, 8, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(0, drawings.len());
    }

    #[test]
    fn given_single_event_when_event_starts_at_end_of_first_day_then_draw_nothing() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 1, 18, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 1, 22, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(0, drawings.len());
    }

    #[test]
    fn given_single_event_when_event_ends_at_start_of_second_day_then_draw_nothing() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 2, 6, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 2, 8, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(0, drawings.len());
    }

    #[test]
    fn given_single_event_when_event_starts_at_end_of_second_day_then_draw_nothing() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 2, 18, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 2, 22, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(0, drawings.len());
    }

    #[test]
    fn given_single_event_when_event_spans_both_days_then_draw_two_rectangles() {
        let events = vec![CalendarEvent {
            name: "foo".to_string(),
            times: IndefiniteTimeRange {
                start: Some(Local.with_ymd_and_hms(2022, 9, 1, 12, 0, 0).unwrap()),
                end: Some(Local.with_ymd_and_hms(2022, 9, 2, 12, 0, 0).unwrap()),
            },
        }];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(2, drawings.len());
        assert_eq!(draw::Point { x: 0.0, y: 40.0 }, drawings[0].position);
        assert!(matches!(
            drawings[0].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 60
            })
        ));
        assert_eq!(draw::Point { x: 100.0, y: 0.0 }, drawings[1].position);
        assert!(matches!(
            drawings[1].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 40
            })
        ));
    }

    #[test]
    fn given_two_events_when_events_are_disjoint_then_draw_two_fullwidth_rectangles() {
        let events = vec![
            CalendarEvent {
                name: "foo".to_string(),
                times: IndefiniteTimeRange {
                    start: Some(Local.with_ymd_and_hms(2022, 9, 1, 12, 0, 0).unwrap()),
                    end: Some(Local.with_ymd_and_hms(2022, 9, 1, 14, 0, 0).unwrap()),
                },
            },
            CalendarEvent {
                name: "foo".to_string(),
                times: IndefiniteTimeRange {
                    start: Some(Local.with_ymd_and_hms(2022, 9, 2, 12, 0, 0).unwrap()),
                    end: Some(Local.with_ymd_and_hms(2022, 9, 2, 14, 0, 0).unwrap()),
                },
            },
        ];
        let drawer = test_drawer();

        let drawings = drawer.draw(&events, &TEST_BOUNDS);
        assert_eq!(2, drawings.len());
        assert_eq!(draw::Point { x: 0.0, y: 40.0 }, drawings[0].position);
        assert!(matches!(
            drawings[0].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 20
            })
        ));
        assert_eq!(draw::Point { x: 100.0, y: 40.0 }, drawings[1].position);
        assert!(matches!(
            drawings[1].shape,
            Some(Shape::Rectangle {
                width: 100,
                height: 20
            })
        ));
    }

    // TODO: add tests for overlapping events
}
