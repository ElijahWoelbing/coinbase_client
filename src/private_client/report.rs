/// A structure that represents a Report
#[derive(serde::Serialize, Debug)]
pub struct Report {
    r#type: String,
    start_date: String,
    end_date: String,
    product_id: Option<String>,
    account_id: Option<String>,
    format: Format,
    email: Option<String>,
}

impl Report {
    /// Creates a `ReportBuilder` for type fills
    pub fn fills_builder(
        start_date: &str,
        end_date: &str,
        product_id: &str,
    ) -> impl SharedReportOptions + FillsReportOptions {
        ReportBuilder {
            r#type: "fills".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: Some(product_id.to_string()),
            account_id: None,
            format: Format::PDF,
            email: None,
        }
    }

    /// Creates a `ReportBuilder` for type account
    pub fn account_builder(
        start_date: &str,
        end_date: &str,
        account_id: &str,
    ) -> impl SharedReportOptions + AccountReportOptions {
        ReportBuilder {
            r#type: "account".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: None,
            account_id: Some(account_id.to_string()),
            format: Format::PDF,
            email: None,
        }
    }
}

/// A `ReportBuilder` can be used to create a `Report` with custom configuration.
#[derive(Debug)]
pub struct ReportBuilder {
    r#type: String,
    start_date: String,
    end_date: String,
    product_id: Option<String>,
    account_id: Option<String>,
    format: Format,
    email: Option<String>,
}

impl ReportBuilder {
    /// Creates a `ReportBuilder` for type fills
    pub fn fills(
        start_date: &str,
        end_date: &str,
        product_id: &str,
    ) -> impl SharedReportOptions + FillsReportOptions {
        Self {
            r#type: "fills".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: Some(product_id.to_string()),
            account_id: None,
            format: Format::PDF,
            email: None,
        }
    }

    /// Creates a `ReportBuilder` for type account
    pub fn account(
        start_date: &str,
        end_date: &str,
        account_id: &str,
    ) -> impl SharedReportOptions + AccountReportOptions {
        Self {
            r#type: "account".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: None,
            account_id: Some(account_id.to_string()),
            format: Format::PDF,
            email: None,
        }
    }
}

/// Fills only builder options
pub trait FillsReportOptions {
    fn account_id(self, account_id: &str) -> Self;
}

impl FillsReportOptions for ReportBuilder {
    /// ID of the account to generate an account report for
    fn account_id(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.to_string());
        self
    }
}

/// Account only builder options
pub trait AccountReportOptions {
    fn product_id(self, product_id: &str) -> Self;
}

impl AccountReportOptions for ReportBuilder {
    /// ID of the product to generate a fills report for. E.g. BTC-USD
    fn product_id(mut self, product_id: &str) -> Self {
        self.product_id = Some(product_id.to_string());
        self
    }
}

/// Fills and account builder options
pub trait SharedReportOptions {
    fn format(self, format: Format) -> Self;
    fn email(self, email: &str) -> Self;
    fn build(self) -> Report;
}

impl SharedReportOptions for ReportBuilder {
    /// pdf or csv (defualt is pdf)
    fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Email address to send the report to
    fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    /// Builds `Report`
    fn build(self) -> Report {
        Report {
            r#type: self.r#type,
            start_date: self.start_date,
            end_date: self.end_date,
            product_id: self.product_id,
            account_id: self.account_id,
            format: self.format,
            email: self.email,
        }
    }
}

/// Type of report
#[derive(Debug)]
pub enum Format {
    PDF,
    CSV,
}

impl serde::Serialize for Format {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::PDF => serializer.serialize_str("pdf"),
            Self::CSV => serializer.serialize_str("csv"),
        }
    }
}
