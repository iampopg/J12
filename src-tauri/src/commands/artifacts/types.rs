use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomySubcategorySummary {
    pub subcategory_id: String,
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomyDomainSummary {
    pub domain_id: String,
    pub name: String,
    pub icon: String,
    pub total_count: usize,
    pub subcategories: Vec<TaxonomySubcategorySummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForensicTaxonomyArtifact {
    pub id: String,
    pub domain_id: String,
    pub subcategory_id: String,
    pub title: String,
    pub primary_value: String,
    pub secondary_value: Option<String>,
    pub details: String,
    pub severity: String,
    pub artifact_type: String,
    pub confidence: Option<String>,
    pub email_id: String,
    pub email_subject: Option<String>,
    pub email_from: String,
    pub date_sent_utc: Option<String>,
}
