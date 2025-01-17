use std::process::exit;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Weekday};

enum TimeUnit {
    Day,
    RelativeDay,
    RelativeWeek,
    RelativeMonth,
    RelativeYear,
}

enum TimeWeekDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug)]
pub struct TaigaTime {
    pub date: NaiveDate,
}

impl TaigaTime {
    pub fn format(&self) -> String {
        return self.date.format("%Y-%m-%d").to_string();
    }

    pub fn new(date: String) -> Self {
        if let Ok(date) = NaiveDate::parse_from_str(&date, "%d/%m/%Y") {
            return TaigaTime { date };
        } else if let Ok(date) = NaiveDate::parse_from_str(&date, "%d-%m-%Y") {
            return TaigaTime { date };
        } else if let Ok(date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            return TaigaTime { date };
        } else if let Ok(date) = NaiveDate::parse_from_str(&date, "%Y/%m/%d") {
            return TaigaTime { date };
        }

        let now = Local::now();

        match date.to_lowercase().as_str() {
            "now" | "today" => return TaigaTime { date: now.date_naive() },
            "yes" | "yesterday" => return TaigaTime { date: (now - Duration::days(1)).date_naive() },
            "tom" | "tomorrow" => return TaigaTime { date: (now + Duration::days(1)).date_naive() },
            _ => {}
        }

        if let Some((number, unit)) = Self::parse_ordinal_time(&date) {
            match unit {
                TimeUnit::Day => return TaigaTime { date: Self::find_next(number as u32) },
                TimeUnit::RelativeDay => return TaigaTime { date: (now + Duration::days(number as i64)).date_naive() },
                TimeUnit::RelativeWeek => return TaigaTime { date: (now + Duration::days(7 * number as i64)).date_naive() },
                TimeUnit::RelativeMonth => return TaigaTime { date: (now + Duration::days(30 * number as i64)).date_naive() },
                TimeUnit::RelativeYear => return TaigaTime { date: (now + Duration::days(365 * number as i64)).date_naive() },
            };
        }

        match date.to_lowercase().as_str() {
            "sow" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Monday) },
            "soww" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Monday) },
            "som" => return TaigaTime { date: Self::start_of_next_month(now) },
            "soq" => return TaigaTime { date: Self::start_of_next_quarter(now) },
            "soy" => return TaigaTime { date: Self::start_of_next_year(now) },
            "eow" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Monday) },
            "eoww" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Saturday) },
            "eom" => return TaigaTime { date: Self::end_of_current_month(now) },
            "eoq" => return TaigaTime { date: Self::end_of_current_quarter(now) },
            "eoy" => return TaigaTime { date: Self::end_of_current_year(now) },
            "mon" | "monday" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Monday) },
            "tue" | "tuesday" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Tuesday) },
            "wed" | "wednesday" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Wednesday) },
            "thu" | "thursday" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Thursday) },
            "fri" | "friday" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Friday) },
            "sat" | "saturday" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Saturday) },
            "sun" | "sunday" => return TaigaTime { date: Self::find_next_weekday(TimeWeekDay::Sunday) },
            _ => {},
        }

        eprintln!("Error, could not parse this due date");
        exit(1);
    }

    fn parse_ordinal_time(date: &str) -> Option<(i32, TimeUnit)> {
        let chars = date.chars();
        let mut digit = String::new();
        let mut ordinal = String::new();

        for char in chars {
            if char.is_ascii_digit() || char == '-' {
                digit.push(char);
            } else {
                ordinal.push(char);
            }
        }

        let num = match digit.parse::<i32>() {
            Ok(num) => num,
            Err(_) => return None,
        };

        let time_unit = match ordinal.as_str() {
            "st" => {
                if num == 1 {
                    TimeUnit::Day
                } else {
                    return None;
                }
            },
            "nd" => {
                if num == 2 {
                    TimeUnit::Day
                } else {
                    return None;
                }
            },
            "rd" => {
                if num == 3 {
                    TimeUnit::Day
                } else {
                    return None;
                }
            },
            "th" => {
                if (4..=31).contains(&num) {
                    TimeUnit::Day
                } else {
                    return None;
                }
            },
            "d" | "day" | "days" => TimeUnit::RelativeDay,
            "w" | "wk" | "wks" | "week" | "weeks" => TimeUnit::RelativeWeek,
            "m" | "mth" | "mths" | "month" | "months" => TimeUnit::RelativeMonth,
            "y" | "yr" | "yrs" | "year" | "years" => TimeUnit::RelativeYear,
            _ => return None,
        };

        Some((num, time_unit))
    }

    fn find_next_weekday(weekday: TimeWeekDay) -> NaiveDate {
        let now = Local::now();
        let today_weekday = now.weekday();
        let target_weekday = match weekday {
            TimeWeekDay::Monday => Weekday::Mon,
            TimeWeekDay::Tuesday => Weekday::Tue,
            TimeWeekDay::Wednesday => Weekday::Wed,
            TimeWeekDay::Thursday => Weekday::Thu,
            TimeWeekDay::Friday => Weekday::Fri,
            TimeWeekDay::Saturday => Weekday::Sat,
            TimeWeekDay::Sunday => Weekday::Sun,
        };

        let mut days_until_target = target_weekday.num_days_from_monday() as i64 - today_weekday.num_days_from_monday() as i64;
        if days_until_target <= 0 {
            days_until_target += 7;
        }

        now.date_naive() + chrono::Duration::days(days_until_target)
    }

    fn start_of_next_month(now: DateTime<Local>) -> NaiveDate {
        let mut year = now.year();
        let mut month = now.month();
        month += 1;
        if month > 12 {
            year += 1;
            month = 1;
        }
        NaiveDate::from_ymd_opt(year, month, 1).expect("Could not get next month")
    }

    fn start_of_next_quarter(now: DateTime<Local>) -> NaiveDate {
        let mut month = now.month();
        month = ((((month - 1) / 3) + 1) % 4) * 3 + 1;
        let year = now.year() + if now.month() > 9 { 1 } else { 0 };
        NaiveDate::from_ymd_opt(year, month, 1).expect("Could not get next quarter")
    }

    fn start_of_next_year(now: DateTime<Local>) -> NaiveDate {
        NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).expect("Could not get next year")
    }

    fn end_of_current_month(now: DateTime<Local>) -> NaiveDate {
        let mut year = now.year();
        let mut month = now.month();
        month += 1;
        if month > 12 {
            year += 1;
            month = 1;
        }
        NaiveDate::from_ymd_opt(year, month, 1).expect("Could not get last year")  - Duration::days(1)
    }

    fn end_of_current_quarter(now: DateTime<Local>) -> NaiveDate {
        let mut month = now.month();
        month = ((((month - 1) / 3) + 1) % 4) * 3 + 1;
        let year = now.year() + if now.month() > 9 { 1 } else { 0 };
        NaiveDate::from_ymd_opt(year, month, 1).expect("Could not get next quarter") - Duration::days(1)
    }

    fn end_of_current_year(now: DateTime<Local>) -> NaiveDate {
        NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).expect("Could not get next year") - Duration::days(1)
    }

    fn find_next(to_day: u32) -> NaiveDate {
        let now = Local::now();
        let mut month = now.month();
        let mut year = now.year();

        if now.day() >= to_day {
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }

        loop {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, to_day) {
                return date;
            }
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
}
