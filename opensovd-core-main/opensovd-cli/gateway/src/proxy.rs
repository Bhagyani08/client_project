use async_trait::async_trait;
use opensovd_core::{DataProvider, DataFilter, Metadata, Data, DataError};
use opensovd_client::Client;

pub struct HttpProxyProvider {
    client: Client,
    component_id: String,
}

impl HttpProxyProvider {
    pub fn new(base_uri: &str, component_id: &str) -> anyhow::Result<Self> {
        let client = Client::builder()
            .base_uri(base_uri)
            .map_err(|e| anyhow::anyhow!(e))?
            .build()
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(Self {
            client,
            component_id: component_id.to_string(),
        })
    }
}

#[async_trait]
impl DataProvider for HttpProxyProvider {
    async fn read(&self, data_id: &str, include_schema: bool) -> Result<Data, DataError> {
        let response = self.client
            .component(&self.component_id)
            .data(data_id)
            .read()
            .schema(include_schema)
            .send()
            .await
            .map_err(|e| DataError::Internal(e.to_string()))?;

        Ok(Data {
            data: response.data,
            schema: response.schema,
        })
    }

    async fn read_with_authorization(
        &self,
        data_id: &str,
        include_schema: bool,
        authorization: Option<&str>,
    ) -> Result<Data, DataError> {
        let component = self.client.component(&self.component_id);
        let data = component.data(data_id);
        let mut request = data.read().schema(include_schema);
        if let Some(authorization) = authorization {
            request = request.authorization(authorization);
        }
        let response = request
            .send()
            .await
            .map_err(|e| DataError::Internal(e.to_string()))?;

        Ok(Data {
            data: response.data,
            schema: response.schema,
        })
    }

    async fn write(&self, data_id: &str, value: serde_json::Value) -> Result<(), DataError> {
        self.client
            .component(&self.component_id)
            .data(data_id)
            .write(&value)
            .map_err(|e| DataError::Internal(e.to_string()))?
            .send()
            .await
            .map_err(|e| DataError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn list(&self, _filter: DataFilter) -> Result<Vec<Metadata>, DataError> {
        // Simplified mapping for proxy listing
        let response = self.client
            .component(&self.component_id)
            .list_data()
            .send()
            .await
            .map_err(|e| DataError::Internal(e.to_string()))?;

        Ok(response.data.items.into_iter().map(|item| Metadata {
            id: item.id.clone(),
            name: item.name.clone(),
            category: "DynamicProxy".to_string(),
            translation_id: None,
            groups: vec![],
            tags: vec![],
            schema: None,
            is_readable: true,
            is_writable: true,
        }).collect())
    }
}
