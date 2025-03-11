use kube::{
    api::{Patch, PatchParams},
    core::DynamicObject,
    discovery::ApiResource,
    Api, Client,
};
use serde_json::json;

use crate::get_config;

pub async fn patch_resource_status(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    name: &str,
    payload: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let status = json!({ "status": payload });
    let patch_params = PatchParams::default();
    api.patch_status(name, &patch_params, &Patch::Merge(status))
        .await?;
    Ok(())
}

pub fn build_hostname(key: &str) -> (String, String) {
    let config = get_config();

    let extension_domain = &config.extension_domain;
    let hostname = extension_domain.clone();
    let hostname_key = format!("{key}.{extension_domain}");

    (hostname, hostname_key)
}
