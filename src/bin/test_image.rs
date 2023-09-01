use cast_schedule::{draw::calendar::*, draw::*, model::*};
use chrono::{prelude::*, *};
use draw::{render::bitmap::PngRenderer, render::save, *};
use std::path::PathBuf;
use tempdir::TempDir;

fn main() {
    let tempdir = TempDir::new("cast").expect("could not create tempdir");
    let image_path = create_image(&tempdir).expect("could not create image");
    let _ = open::that(format!("file://{}", image_path.display()))
        .expect("could not open image with default program");

    std::thread::sleep(std::time::Duration::from_secs(30));

    drop(tempdir);
}

fn midnight_today() -> DateTime<Local> {
    Local::now()
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_local_timezone(Local)
        .unwrap()
}

fn create_image(tempdir: &TempDir) -> std::io::Result<Box<PathBuf>> {
    let drawer = CalendarDrawer::new(DefiniteTimeRange {
        start: midnight_today() + Duration::hours(8),
        end: midnight_today() + Duration::hours(18) + Duration::days(1),
    });
    let model = mock_model();
    let mut canvas = Canvas::new(720, 480);
    let bounds = DrawingBounds {
        left: 0,
        top: 0,
        height: 480,
        width: 720,
    };
    let mut drawings = drawer.draw(&model.events, &bounds);
    canvas.display_list.add(
        Drawing::new()
            .with_shape(Shape::Rectangle {
                width: 720,
                height: 480,
            })
            .with_style(Style {
                fill: Some(Fill {
                    color: RGB::new(255, 255, 255),
                }),
                stroke: Some(Stroke {
                    width: 5,
                    color: RGB::new(0, 0, 0),
                }),
            }),
    );
    canvas.display_list.drawings.append(&mut drawings);
    let path = tempdir.path().join("image.png");
    save(&canvas, path.to_str().unwrap(), PngRenderer::new())?;
    Ok(Box::new(path))
}

fn mock_model() -> Model {
    let now = Local::now();
    let today = now.date_naive();

    Model {
        status: CurrentStatus {
            has_meeting: true,
            mic_active: true,
            in_meeting: true,
        },
        events: vec![
            CalendarEvent {
                name: "Brunch".to_string(),
                times: IndefiniteTimeRange {
                    start: Some(
                        today
                            .and_hms_opt(9, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap(),
                    ),
                    end: Some(
                        today
                            .and_hms_opt(10, 30, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap(),
                    ),
                },
            },
            CalendarEvent {
                name: "Reading".to_string(),
                times: IndefiniteTimeRange {
                    start: Some(
                        today
                            .and_hms_opt(13, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap(),
                    ),
                    end: Some(
                        today
                            .and_hms_opt(13, 45, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap(),
                    ),
                },
            },
        ],
    }
}
