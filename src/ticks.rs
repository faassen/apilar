use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct Ticks(pub u64);

impl fmt::Display for Ticks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn ticks_value_parser(s: &str) -> Result<Ticks, String> {
    let ticks: u64 = s
        .parse()
        .map_err(|_| format!("`{}` isn't a valid ticks", s))?;
    Ok(Ticks(ticks))
}

impl Ticks {
    pub fn parse(s: &str) -> Result<Ticks, String> {
        let ticks: u64 = s
            .parse()
            .map_err(|_| format!("`{}` isn't a valid ticks", s))?;
        Ok(Ticks(ticks))
    }

    pub fn tick(&self) -> Ticks {
        Ticks(self.0.wrapping_add(1))
    }

    pub fn is_at(&self, ticks: Ticks) -> bool {
        self.0 % ticks.0 == 0
    }
}
