#[cfg(test)]
mod tests {
    use crate::domain::models::IngestionConfigRule;
    use regex::Regex;
    use serde_json::json;

    fn create_test_rules() -> Vec<IngestionConfigRule> {
        vec![
            IngestionConfigRule {
                pattern: ".*\\.csv$".to_string(),
                target_table: "csv_data".to_string(),
                parser_config: None,
            },
            IngestionConfigRule {
                pattern: ".*test_no_headers\\.csv$".to_string(),
                target_table: "csv_no_headers_data".to_string(),
                parser_config: Some(json!({"headers": ["name", "age", "email", "city"]})),
            },
            IngestionConfigRule {
                pattern: "reports/.*\\.xlsx$".to_string(),
                target_table: "excel_reports".to_string(),
                parser_config: None,
            },
        ]
    }

    fn find_best_match<'a>(s3_key: &str, rules: &'a [IngestionConfigRule]) -> Option<&'a IngestionConfigRule> {
        let matching_rules: Vec<_> = rules
            .iter()
            .filter(|rule| {
                Regex::new(&rule.pattern).unwrap().is_match(s3_key)
            })
            .collect();

        matching_rules
            .into_iter()
            .max_by_key(|rule| rule.pattern.len())
    }

    #[test]
    fn test_specific_pattern_wins() {
        let rules = create_test_rules();
        let result = find_best_match("data/test_no_headers.csv", &rules).unwrap();
        
        assert_eq!(result.target_table, "csv_no_headers_data");
        assert_eq!(result.pattern, ".*test_no_headers\\.csv$");
    }

    #[test]
    fn test_general_pattern_fallback() {
        let rules = create_test_rules();
        let result = find_best_match("data/regular.csv", &rules).unwrap();
        
        assert_eq!(result.target_table, "csv_data");
        assert_eq!(result.pattern, ".*\\.csv$");
    }

    #[test]
    fn test_no_match() {
        let rules = create_test_rules();
        let result = find_best_match("data/file.txt", &rules);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_excel_reports_match() {
        let rules = create_test_rules();
        let result = find_best_match("reports/monthly.xlsx", &rules).unwrap();
        
        assert_eq!(result.target_table, "excel_reports");
    }
}