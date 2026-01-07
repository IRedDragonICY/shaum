use chrono::{NaiveDate, Datelike};
use crate::RuleContext;
use shaum_types::FastingStatus;

/// Iterator for Daud fasting days.
pub struct DaudIterator<'a> {
    current: NaiveDate,
    context: &'a RuleContext,
    is_fasting_turn: bool,
}

impl<'a> DaudIterator<'a> {
    pub fn new(start: NaiveDate, context: &'a RuleContext) -> Self {
        Self {
            current: start,
            context,
            is_fasting_turn: true, // Start with fasting unless configured otherwise
        }
    }
}

impl Iterator for DaudIterator<'_> {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Safety break 
            if self.current.year() > 2100 { return None; }

            let date = self.current;
            self.current = self.current.succ_opt()?;

            // Check if Haram
            let analysis = crate::check(date, self.context).ok()?;
            use shaum_types::DaudStrategy;
            
            if analysis.primary_status == FastingStatus::Haram {
                // Formatting Note: Haram means we MUST NOT fast.
                if self.is_fasting_turn {
                    // It was our turn to fast.
                    match self.context.daud_strategy {
                        DaudStrategy::Skip => {
                            // Skip this turn entirely. Next day is Eat day.
                            self.is_fasting_turn = false;
                        },
                        DaudStrategy::Postpone => {
                            // Postpone this turn. Next day we try to fast again (keep state true).
                            // self.is_fasting_turn = true; (unchanged)
                        }
                    }
                    continue;
                } else {
                    // It was our turn to eat. Haram enforces eating. Matches pattern.
                    // Move to next turn (Fast).
                    self.is_fasting_turn = true;
                    continue;
                }
            }

            if self.is_fasting_turn {
                self.is_fasting_turn = false;
                return Some(date);
            } else {
                self.is_fasting_turn = true;
                continue;
            }
        }
    }
}

/// Generates a list of Daud fasting days between start and end (inclusive).
pub fn generate_daud_schedule(
    start: NaiveDate,
    end: NaiveDate,
    context: &RuleContext
) -> Vec<NaiveDate> {
    DaudIterator::new(start, context)
        .take_while(|d| *d <= end)
        .collect()
}

/// Builder for Daud fasting schedule.
pub struct DaudScheduleBuilder {
    start: NaiveDate,
    end: Option<NaiveDate>,
    postpone_on_haram: bool,
    context: RuleContext,
}

impl DaudScheduleBuilder {
    pub fn new(start: NaiveDate) -> Self {
        Self {
            start,
            end: None,
            postpone_on_haram: false,
            context: RuleContext::default(),
        }
    }

    pub fn until(mut self, end: NaiveDate) -> Self {
        self.end = Some(end);
        self
    }

    pub fn postpone_on_haram(mut self) -> Self {
        self.postpone_on_haram = true;
        self
    }

    pub fn skip_haram_days(mut self) -> Self {
        self.postpone_on_haram = false;
        self
    }

    pub fn with_context(mut self, ctx: RuleContext) -> Self {
        self.context = ctx;
        self
    }

    pub fn build(self) -> Vec<Result<NaiveDate, shaum_types::ShaumError>> {
        let mut results = Vec::new();
        // Use DaudIterator logic
        let iter = DaudIterator::new(self.start, &self.context);
        
        let end = self.end.unwrap_or_else(|| self.start.checked_add_signed(chrono::Duration::days(365)).unwrap());
        
        // TODO: Implement postpone logic properly if needed.
        // For now, simple wrapper to satisfy API.
        for date in iter.take_while(|d| *d <= end) {
             results.push(Ok(date));
        }
        results
    }
}
