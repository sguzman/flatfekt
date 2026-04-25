use flatfekt_runtime::TimelineClock;
use flatfekt_runtime::animation::{
  PlannedEvent,
  TimelinePlan,
  apply_seek_timeline
};

#[test]
fn timeline_seek_sets_cursor_deterministically()
 {
  let mut clock = TimelineClock {
    enabled: true,
    playing: true,
    t_secs: 0.0,
    dt_secs: 1.0 / 60.0,
    ..Default::default()
  };

  let mut plan = TimelinePlan {
    events: vec![
      PlannedEvent {
        time:    0.5,
        action:  "noop".to_owned(),
        target:  None,
        payload: None,
        track:   None,
        label:   None,
        group:   None,
        after:   None
      },
      PlannedEvent {
        time:    1.0,
        action:  "noop".to_owned(),
        target:  None,
        payload: None,
        track:   None,
        label:   None,
        group:   None,
        after:   None
      },
      PlannedEvent {
        time:    2.0,
        action:  "noop".to_owned(),
        target:  None,
        payload: None,
        track:   None,
        label:   None,
        group:   None,
        after:   None
      },
    ],
    cursor: 0
  };

  apply_seek_timeline(
    &mut clock, &mut plan, 0.8, 0.8,
    false
  )
  .expect("seek applies");

  assert!(
    (clock.t_secs - 0.8).abs() < 1e-6
  );
  assert!(!clock.playing);
  assert_eq!(plan.cursor, 1);
}
