

/// The legal status (Hukum) of fasting on a specific day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FastingStatus {
    /// Permissible (Neutral). Default status for most days.
    Mubah,
    /// Strongly Recommended (e.g., Arafah, Ashura).
    SunnahMuakkadah,
    /// Recommended (e.g., Monday/Thursday).
    Sunnah,
    /// Disliked (e.g., Singling out Friday).
    Makruh,
    /// Prohibited (e.g., Eid dates, Tashriq).
    Haram,
    /// Obligatory (Ramadhan).
    Wajib,
}

impl FastingStatus {
    pub fn is_haram(&self) -> bool {
        matches!(self, FastingStatus::Haram)
    }

    pub fn is_wajib(&self) -> bool {
        matches!(self, FastingStatus::Wajib)
    }

    pub fn is_sunnah(&self) -> bool {
        matches!(self, FastingStatus::Sunnah | FastingStatus::SunnahMuakkadah)
    }
}

/// The specific reason or type of fasting associated with a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastingType {
    Ramadhan,
    Arafah,
    Tasua,
    Ashura,
    AyyamulBidh,
    Monday,
    Thursday,
    Shawwal,
    Daud,
    EidAlFitr,
    EidAlAdha,
    Tashriq,
    FridayExclusive,
    SaturdayExclusive,
}

#[derive(Debug, Clone)]
pub struct FastingAnalysis {
    pub date: chrono::NaiveDate,
    pub primary_status: FastingStatus,
    pub types: Vec<FastingType>,
    pub description: String,
}

impl FastingAnalysis {
    pub fn new(date: chrono::NaiveDate, status: FastingStatus, types: Vec<FastingType>) -> Self {
        Self {
            date,
            primary_status: status,
            types,
            description: String::new(),
        }
    }
    
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}
