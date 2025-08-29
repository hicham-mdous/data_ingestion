#[cfg(test)]
mod tests {
    use crate::infrastructure::parsers::csv_parser::{parse_csv, parse_csv_with_config};
    use serde_json::json;

    #[test]
    fn test_csv_with_headers() {
        let csv_data = b"name,age,email\nJohn,25,john@test.com\nJane,30,jane@test.com";
        let result = parse_csv(csv_data).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["name"], "John");
        assert_eq!(result[0]["age"], "25");
        assert_eq!(result[1]["name"], "Jane");
    }

    #[test]
    fn test_csv_without_headers() {
        let csv_data = b"John,25,john@test.com\nJane,30,jane@test.com";
        let config = json!({"headers": ["name", "age", "email"]});
        let result = parse_csv_with_config(csv_data, Some(&config)).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["name"], "John");
        assert_eq!(result[0]["age"], "25");
        assert_eq!(result[1]["name"], "Jane");
    }

    #[test]
    fn test_csv_fallback_columns() {
        let csv_data = b"John,25,john@test.com,extra\nJane,30,jane@test.com,data";
        let config = json!({"headers": ["name", "age"]});
        let result = parse_csv_with_config(csv_data, Some(&config)).unwrap();
        
        assert_eq!(result[0]["name"], "John");
        assert_eq!(result[0]["column_2"], "john@test.com");
        assert_eq!(result[0]["column_3"], "extra");
    }
}