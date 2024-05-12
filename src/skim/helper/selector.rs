use std::collections::HashSet;

use regex::Regex;

use crate::skim::{Selector, SkimItem};

#[derive(Debug, Default)]
pub struct DefaultSkimSelector {
    first_n: usize,
    regex: Option<Regex>,
    preset: Option<HashSet<String>>,
}

impl Selector for DefaultSkimSelector {
    fn should_select(&self, index: usize, item: &dyn SkimItem) -> bool {
        if self.first_n > index {
            return true;
        }

        if self.preset.is_some()
            && self
                .preset
                .as_ref()
                .map(|preset| preset.contains(item.text().as_ref()))
                .unwrap_or(false)
        {
            return true;
        }

        if self.regex.is_some()
            && self
                .regex
                .as_ref()
                .map(|re| re.is_match(&item.text()))
                .unwrap_or(false)
        {
            return true;
        }

        false
    }
}
