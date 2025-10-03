use crate::models::{JobBaseResource, JobBaseResourceArmPaginatedResult};
use async_trait::async_trait;
use azure_core::{http::pager::Page, Result};

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Page for JobBaseResourceArmPaginatedResult {
    type Item = JobBaseResource;
    type IntoIter = <Vec<JobBaseResource> as IntoIterator>::IntoIter;
    async fn into_items(self) -> Result<Self::IntoIter> {
        // TODO: Deal with this better
        Ok(self.value.expect("Expected list").into_iter())
    }
}
