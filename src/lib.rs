// https://github.com/azza-bazoo/prettycron/blob/master/prettycron.js

extern crate cron;
extern crate inflector;

use cron::{Schedule, TimeUnitSpec};
use std::str::FromStr;
use inflector::InflectorNumbers;

/// For an array of numbers, e.g. a list of hours in a schedule,
/// return a string listing out all of the values (complete with
/// "and" plus ordinal text on the last item).
fn number_list<T: TimeUnitSpec>(numbers: &T) -> String {
    if numbers.count() < 2 {
        return format!("{}", numbers.iter().nth(0).unwrap().ordinalize());
    }

    let mut nums: Vec<_> = numbers.iter().collect();
    let last_val = nums.pop().unwrap();
    format!("{} and {}",
            nums.into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            last_val.ordinalize())
}

fn step_size<T: TimeUnitSpec>(numbers: &T) -> usize {
    if numbers.count() <= 1 {
        return 0;
    }

    let expected_step = numbers.iter().nth(1).unwrap() - numbers.iter().nth(0).unwrap();
    if numbers.count() == 2 {
        return expected_step as usize;
    }

    0
    // Check that every number is the previous number + the first number
    //return numbers.slice(1).every(function(n,i,a){
    //  return (i === 0 ? n : n-a[i-1]) === expected_step;
    //}) ? expected_step : 0;
}

fn is_every_other<T: TimeUnitSpec>(step: usize, numbers: &T) -> bool {
    numbers.count() == 30 && step == 2
}

fn is_twice_per_hour<T: TimeUnitSpec>(step: usize, numbers: &T) -> bool {
    numbers.count() == 2 && step == 30
}

fn is_on_the_hour<T: TimeUnitSpec>(numbers: &T) -> bool {
    numbers.count() == 1 && numbers.iter().nth(0).unwrap() == 0
}

fn is_step_value<T: TimeUnitSpec>(step: usize, numbers: &T) -> bool {
    // Value with slash (https://en.wikipedia.org/wiki/Cron#Non-Standard_Characters)
    numbers.count() > 2 && step > 0
}

/// For an array of numbers of seconds, return a string
/// listing all the values unless they represent a frequency divisible by 60:
/// /2, /3, /4, /5, /6, /10, /12, /15, /20 and /30
fn get_minutes_text_parts<T: TimeUnitSpec>(minutes: &T, star: bool) -> (String, String) {
    if star {
        return ("minute".to_string(), "".to_string());
    }

    let step = step_size(minutes);
    if is_on_the_hour(minutes) {
        ("".to_string(), "hour, on the hour".to_string())
    } else if is_every_other(step, minutes) {
        ("other minute".to_string(), "".to_string())
    } else if is_step_value(step, minutes) {
        ("".to_string(), format!("{} minutes", step))
    } else if is_twice_per_hour(step, minutes) {
        ("".to_string(), "first and 30th minute".to_string())
    } else {
        ("".to_string(), format!("{} minute", number_list(minutes)))
    }
}

/// For an array of numbers of seconds, return a string
/// listing all the values unless they represent a frequency divisible by 60:
/// /2, /3, /4, /5, /6, /10, /12, /15, /20 and /30
fn get_seconds_text_parts<T: TimeUnitSpec>(numbers: &T, star: bool) -> (String, String) {
    if star {
        return ("second".to_string(), "".to_string());
    }

    let step = step_size(numbers);
    if is_every_other(step, numbers) {
        ("".to_string(), "other second".to_string())
    } else if is_step_value(step, numbers) {
        ("".to_string(), format!("{} seconds", step))
    } else {
        ("minute".to_string(),
         format!("starting on the {}",
                 if numbers.count() == 2 && step == 30 {
                     "first and 30th second".to_string()
                 } else {
                     format!("{} second", number_list(numbers))
                 }))
    }
}

/// Parse a number into day of week, or a month name;
/// used in date_list below.
#[derive(Copy, Clone)]
enum DateNaming {
    DOW,
    MON,
}

fn number_to_date_name(value: u32, kind: DateNaming) -> String {
    match kind {
        DateNaming::DOW => {
            format!("DAY({})", value - 1)
            //TODO return moment().day(value - 1).format('ddd')
        }
        DateNaming::MON => format!("MONTH({})", value - 1),
    }
}

/// From an array of numbers corresponding to dates (given in type: either
/// days of the week, or months), return a string listing all the values.
fn date_list<T: TimeUnitSpec>(numbers: &T, kind: DateNaming) -> String {
    let mut values: Vec<_> = numbers.iter().collect();

    if values.len() < 2 {
        return number_to_date_name(values[0], kind);
    }

    let last_val = values.pop().unwrap();
    let mut output_text = "".to_string();

    for item in values {
        if output_text.len() > 0 {
            output_text.push_str(", ");
        }
        output_text.push_str(&number_to_date_name(item, kind));
    }
    format!("{} and {}",
            output_text,
            number_to_date_name(last_val, kind))
}

/// Given a schedule from later.js (i.e. after parsing the cronspec),
/// generate a friendly sentence description.
pub fn prettify_cron(expression: &str) -> String {
    let schedule = Schedule::from_str(expression).unwrap();

    let parts = expression.trim().split_whitespace().collect::<Vec<_>>();

    let every_second = parts[0] == "*";
    let every_minute = parts[1] == "*";
    let every_hour = parts[2] == "*";
    let every_weekday = parts[3] == "*";
    let every_day_in_month = parts[4] == "*";
    let every_month = parts[5] == "*";

    let one_or_two_seconds_per_minute = !every_second && schedule.seconds().count() <= 2;
    let one_or_two_minutes_per_hour = !every_minute && schedule.minutes().count() <= 2;
    let one_or_two_hours_per_day = !every_hour && schedule.hours().count() <= 2;
    let only_specific_days_of_month = !every_day_in_month &&
                                      schedule.days_of_month().count() != 31;

    let mut text_parts = vec![];

    if one_or_two_hours_per_day && one_or_two_minutes_per_hour && one_or_two_seconds_per_minute {
        // If there are only one or two specified values for
        // hour or minute, print them in HH:MM format, or HH:MM:ss if seconds are used
        // If seconds are not used, later.js returns one element for the seconds (set to zero)
    } else {
        let seconds = get_seconds_text_parts(schedule.seconds(), every_second);
        let minutes = get_minutes_text_parts(schedule.minutes(), every_minute);
        let mut beginning = "".to_string();
        let mut end = "".to_string();

        text_parts.push("Every".to_string());

        // Otherwise, list out every specified hour/minute value.
        let has_specific_seconds = !every_second &&
                                   ((schedule.seconds().count() > 1 &&
                                     schedule.seconds().count() < 60) ||
                                    (schedule.seconds().count() == 1 &&
                                     schedule.seconds().iter().nth(0).unwrap() != 0));
        if has_specific_seconds {
            beginning = seconds.0.to_string();
            end = seconds.1.to_string();
        }

        if !every_hour {
            if has_specific_seconds {
                end.push_str(" on the ");
            }
            if !every_minute {
                // and only at specific minutes
                let hours = format!("{} hour", number_list(schedule.hours()));
                if !has_specific_seconds && is_on_the_hour(schedule.minutes()) {
                    text_parts = vec!["On the".to_string()];
                    end.push_str(&hours);
                } else {
                    beginning = minutes.0.to_string();
                    end.push_str(&format!("{} past the {}", minutes.1, hours));
                }
            } else {
                // specific hours, but every minute
                end.push_str(&format!("minute of {} hour", number_list(schedule.hours())));
            }
        } else if !every_minute {
            // every hour, but specific minutes
            beginning = minutes.0.to_string();
            end.push_str(&minutes.1);
            if !is_on_the_hour(schedule.minutes()) &&
               (only_specific_days_of_month || !every_weekday || !every_month) {
                end.push_str(" past every hour");
            }
        } else if every_second && every_minute {
            beginning = seconds.0.to_string();
        } else if !has_specific_seconds {
            beginning.push_str(&minutes.0)
        }

        text_parts.push(beginning);
        text_parts.push(end);
    }

    if only_specific_days_of_month {
        // runs only on specific day(s) of month
        text_parts.push(format!("on the {}", number_list(schedule.days_of_month())));
        if every_month {
            text_parts.push("of every month".into());
        }
    }

    if !every_weekday {
        // runs only on specific day(s) of week
        if !every_day_in_month {
            // if both day fields are specified, cron uses both; superuser.com/a/348372
            text_parts.push("and every".into());
        } else {
            text_parts.push("on".into());
        }
        text_parts.push(date_list(schedule.days_of_week(), DateNaming::DOW));
    }

    if !every_month {
        if schedule.months().count() == 12 {
            text_parts.push("day of every month".into());
        } else {
            // runs only in specific months; put this output last
            text_parts.push(format!("in {}",
                                    date_list(schedule.months(), DateNaming::MON)));
        }
    }

    text_parts
        .into_iter()
        .filter(|x| x.len() > 0)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        //               sec  min   hour   day of month   month   day of week   year
        //let expression = "*   30   9,12,15     1,15       May-Aug  Mon,Wed,Fri  2018/2";
        //let res = super::prettify_cron(expression, true);
        //println!("{:?}", res);

        //let expression = "0 0 18 1/1 * ? * *";
        //let res = super::prettify_cron(expression);
        //assert_eq!(res, "18:00:00 every day");

        let expression = "* * * * * * *";
        let res = super::prettify_cron(expression);
        assert_eq!(res, "Every second");

        //let expression = "0/1 0/1 0/1 0/1 0/1 0/1 *";
        //let res = super::prettify_cron(expression);
        //assert_eq!(res, "Every second");

        //let expression = "*/4 2 4 * * * *";
        //let res = super::prettify_cron(expression);
        //assert_eq!(res, "Every second");

        //let expression = "30 15 9 * * * *";
        //let res = super::prettify_cron(expression);
        //println!("{:?}", res);
        //assert_eq!(res, "09:15:30 every day");

        let expression = "5 * * * * *";
        let res = super::prettify_cron(expression);
        assert_eq!(res, "Every minute starting on the 5th second");

        let expression = "30 * * * * *";
        let res = super::prettify_cron(expression);
        assert_eq!(res, "Every minute starting on the 30th second");

        let expression = "0,2,4,20 * * * * *";
        let res = super::prettify_cron(expression);
        assert_eq!(res, "Every minute starting on the 0, 2, 4 and 20th second");

        let expression = "15-17 * * * * *";
        let res = super::prettify_cron(expression);
        assert_eq!(res, "Every minute starting on the 15, 16 and 17th second");
    }
}
