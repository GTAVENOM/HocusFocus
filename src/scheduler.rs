use std::collections::HashSet;
use chrono::{DateTime, Datelike, Local, NaiveTime, Weekday};
use crate::config::{Rule,Schedule};

fn parse_weekday(day_str: &str) ->Option<Weekday>{
    match day_str.to_lowercase().as_str(){
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    }
}

impl Schedule {
    pub fn start_time(&self) -> Result<NaiveTime, chrono::ParseError> {
        NaiveTime::parse_from_str(&self.start, "%H:%M")
    }
    pub fn end_time(&self) -> Result<NaiveTime, chrono::ParseError> {
        NaiveTime::parse_from_str(&self.end, "%H:%M")
    }
    pub fn is_active_at(&self, dt: &DateTime<Local>) -> bool {
        let start = match self.start_time() {
            Ok(t) => t,
            Err(_) => return false,
        };
        let end = match self.end_time() {
            Ok(t) => t,
            Err(_) => return false,
        };
        let current_weekday = dt.weekday();
        let current_time = dt.time();

        let allowed_weekdays: Vec<Weekday> = self
            .days
            .iter()
            .filter_map(|d| parse_weekday(d))
            .collect();
        let day_matches = allowed_weekdays.is_empty() || allowed_weekdays.contains(&current_weekday);
        if start <= end {
            day_matches && current_time >= start && current_time <= end
        } else {
            let today_matches = allowed_weekdays.is_empty() || allowed_weekdays.contains(&current_weekday);
            let yesterday = current_weekday.pred();
            let yesterday_matches = allowed_weekdays.is_empty() || allowed_weekdays.contains(&yesterday);
            (today_matches && current_time >= start) || (yesterday_matches && current_time <= end)
        }
    }
}
impl Rule {
    pub fn is_active_at(&self, dt: &DateTime<Local>) -> bool {
        self.schedules.iter().any(|s| s.is_active_at(dt))
    }
}
pub fn get_blocked_domains(rules: &[Rule], dt: &DateTime<Local>) -> HashSet<String> {
    let mut blocked = HashSet::new();
    for rule in rules {
        if rule.is_active_at(dt) {
            for domain in &rule.domains {
                blocked.insert(domain.clone());
            }
        }
    }
    blocked
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    fn create_test_schedule(days: &[&str], start: &str, end: &str) -> Schedule {
        Schedule {
            days: days.iter().map(|s| s.to_string()).collect(),
            start: start.to_string(),
            end: end.to_string(),
        }
    }
    #[test]
    fn test_normal_schedule() {
        let schedule = create_test_schedule(&["Mon", "Wed", "Fri"], "09:00", "17:00");
        let monday_10am = Local.with_ymd_and_hms(2026, 6, 8, 10, 0, 0).unwrap();
        assert!(schedule.is_active_at(&monday_10am));
        let monday_6pm = Local.with_ymd_and_hms(2026, 6, 8, 18, 0, 0).unwrap();
        assert!(!schedule.is_active_at(&monday_6pm));
        let tuesday_10am = Local.with_ymd_and_hms(2026, 6, 9, 10, 0, 0).unwrap();
        assert!(!schedule.is_active_at(&tuesday_10am));
    }
    #[test]
    fn test_cross_midnight_schedule() {
        let schedule = create_test_schedule(&["Mon", "Tue"], "22:00", "06:00");
        let monday_11pm = Local.with_ymd_and_hms(2026, 6, 8, 23, 0, 0).unwrap();
        assert!(schedule.is_active_at(&monday_11pm));
        let tuesday_2am = Local.with_ymd_and_hms(2026, 6, 9, 2, 0, 0).unwrap();
        assert!(schedule.is_active_at(&tuesday_2am));
        let wednesday_2am = Local.with_ymd_and_hms(2026, 6, 10, 2, 0, 0).unwrap();
        assert!(schedule.is_active_at(&wednesday_2am));
        let thursday_2am = Local.with_ymd_and_hms(2026, 6, 11, 2, 0, 0).unwrap();
        assert!(!schedule.is_active_at(&thursday_2am));
    }
}