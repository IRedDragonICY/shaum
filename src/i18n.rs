use crate::types::{FastingAnalysis, FastingStatus, FastingType};

pub trait Localizer {
    fn month_name(&self, month: usize) -> String;
    fn status_name(&self, status: FastingStatus) -> String;
    fn type_name(&self, f_type: FastingType) -> String;
    fn format_description(&self, analysis: &FastingAnalysis) -> String;
}

pub struct EnglishLocalizer;

impl Localizer for EnglishLocalizer {
    fn month_name(&self, month: usize) -> String {
        crate::calendar::get_hijri_month_name(month).to_string()
    }

    fn status_name(&self, status: FastingStatus) -> String {
        status.to_string()
    }

    fn type_name(&self, f_type: FastingType) -> String {
        f_type.to_string()
    }

    fn format_description(&self, analysis: &FastingAnalysis) -> String {
        format!(
            "Hijri Date: {} {} {}", 
            analysis.hijri_day, 
            self.month_name(analysis.hijri_month), 
            analysis.hijri_year
        )
    }
}
