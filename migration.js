// MongoDB migration script
db = db.getSiblingDB('ingestion_db');

// Create ingestion_config collection with example rules
db.ingestion_config.insertMany([
  {
    pattern: ".*\\.csv$",
    target_table: "csv_data",
    parser_config: {
      delimiter: ",",
      has_headers: true
    }
  },
  {
    pattern: ".*\\.json$",
    target_table: "json_data",
    parser_config: null
  },
  {
    pattern: "reports/.*\\.xlsx?$",
    target_table: "excel_reports",
    parser_config: {
      sheet_index: 0
    }
  },
  {
    pattern: "logs/.*\\.txt$",
    target_table: "text_logs",
    parser_config: null
  },

  {
    pattern: ".*\\.xml$",
    target_table: "xml_data",
    parser_config: null
  },
  {
    pattern: ".*\\.xlsx?$",
    target_table: "excel_data",
    parser_config: {
      sheet_index: 0
    }
  },
  {
    pattern: ".*test_no_headers\\.csv$",
    target_table: "csv_no_headers_data",
    parser_config: {
      headers: ["name", "age", "email", "city"]
    }
  }
]);

print("Migration completed successfully!");