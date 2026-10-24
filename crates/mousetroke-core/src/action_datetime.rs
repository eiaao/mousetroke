//! 本地时间的纯格式化；平台负责取本地时间，核心不读取时钟或时区。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl LocalDateTime {
    pub fn valid(self) -> bool {
        let leap = self.year.is_multiple_of(4)
            && (!self.year.is_multiple_of(100) || self.year.is_multiple_of(400));
        let days = match self.month {
            2 => {
                if leap {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => return false,
        };
        (1..=9999).contains(&self.year)
            && (1..=days).contains(&self.day)
            && self.hour < 24
            && self.minute < 60
            && self.second <= 60
    }
}

/// ID：date、date_zh、date_us、time、datetime；其他字符串作为自定义模式。
/// token：YYYY、MM、DD、HH、mm、ss，区分大小写，其余字符原样输出。
/// 单引号包围字面文本（例如 'YYYY'），两个单引号输出一个；未闭合引号报错。
pub fn format_now(format_id_or_custom: &str, now: LocalDateTime) -> Result<String, String> {
    if !now.valid() {
        return Err("Invalid local date/time".into());
    }
    let pattern = match format_id_or_custom {
        "date" => "YYYY-MM-DD",
        "date_zh" => "YYYY年MM月DD日", // i18n:allow 日期模式为设计指定的输出内容，不是界面文案。
        "date_us" => "MM/DD/YYYY",
        "time" => "HH:mm",
        "datetime" => "YYYY-MM-DD HH:mm",
        custom => custom,
    };
    let tokens = [
        ("YYYY", format!("{:04}", now.year)),
        ("MM", format!("{:02}", now.month)),
        ("DD", format!("{:02}", now.day)),
        ("HH", format!("{:02}", now.hour)),
        ("mm", format!("{:02}", now.minute)),
        ("ss", format!("{:02}", now.second)),
    ];
    let mut rest = pattern;
    let mut result = String::new();
    let mut quoted = false;
    while !rest.is_empty() {
        if let Some(tail) = rest.strip_prefix("''") {
            result.push('\'');
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix('\'') {
            quoted = !quoted;
            rest = tail;
        } else if let Some((token, value)) = tokens
            .iter()
            .find(|(token, _)| !quoted && rest.starts_with(token))
        {
            result.push_str(value);
            rest = &rest[token.len()..];
        } else {
            let ch = rest.chars().next().unwrap();
            result.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
    }
    if quoted {
        Err("Unclosed quote in date/time pattern".into())
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const NOW: LocalDateTime = LocalDateTime {
        year: 2026,
        month: 10,
        day: 24,
        hour: 14,
        minute: 30,
        second: 7,
    };

    #[test]
    fn design_formats_and_custom_tokens() {
        for (id, expected) in [
            ("date", "2026-10-24"),
            ("date_zh", "2026年10月24日"),
            ("date_us", "10/24/2026"),
            ("time", "14:30"),
            ("datetime", "2026-10-24 14:30"),
            ("YYYY年MM月DD日 HH:mm:ss", "2026年10月24日 14:30:07"),
            ("'YYYY' YYYY '' ✅", "YYYY 2026 ' ✅"),
            ("", ""),
            ("原样 yyyy M d", "原样 yyyy M d"),
        ] {
            assert_eq!(format_now(id, NOW).unwrap(), expected);
        }
        assert!(format_now("'YYYY", NOW).is_err());
    }

    #[test]
    fn padding_leap_days_and_invalid_components() {
        let now = LocalDateTime {
            year: 2000,
            month: 2,
            day: 29,
            hour: 0,
            minute: 1,
            second: 2,
        };
        assert_eq!(
            format_now("YYYY-MM-DD HH:mm:ss", now).unwrap(),
            "2000-02-29 00:01:02"
        );
        for bad in [
            LocalDateTime { year: 1900, ..now },
            LocalDateTime { month: 0, ..now },
            LocalDateTime { day: 0, ..now },
            LocalDateTime {
                month: 4,
                day: 31,
                ..now
            },
            LocalDateTime { hour: 24, ..now },
            LocalDateTime { minute: 60, ..now },
            LocalDateTime { second: 61, ..now },
            LocalDateTime { year: 0, ..now },
        ] {
            assert!(format_now("date", bad).is_err());
        }
    }
}
