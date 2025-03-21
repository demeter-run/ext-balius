# Terraform Module Installation Guide

This document outlines the steps to install and configure the Terraform module located in this repository.

## Prerequisites

Before you begin, ensure you have the following prerequisites:

* **Terraform:** Install Terraform version 0.12 or later.
* **kubectl:** Install kubectl to interact with your Kubernetes cluster.
* **Kubernetes Cluster:** You need a running Kubernetes cluster.
* **kubectl configured:** Your `kubectl` must be configured to point to your Kubernetes cluster.

You must also have a Demeter cluster set up using [Up](https://github.com/demeter-run/up).

## Module Structure

The module consists of the following sub-modules:

* **feature:** Deploys the feature operator along with the necessary components for the feature as a whole.
* **proxy:** Deploys the proxy that handles routing, auth, and rate limiting.
* **instance:** Deploys individual instances based on the provided configuration.
* **service:** Deploys Kubernetes services for each network.

The main `main.tf` file in the root directory orchestrates the deployment of these sub-modules.

## Installation Steps

1.  **Clone the Repository:**
    ```bash
    git clone <repository_url>
    cd <repository_directory>
    ```

2.  **Create a `terraform.tfvars` file:**
    Create a `terraform.tfvars` file in the root directory of the module. This file will contain the variable values for your deployment.

3.  **Configure `terraform.tfvars`:**
    Add the required variables to your `terraform.tfvars` file. Here's an example:

    ```terraform
    namespace = "my-namespace"
    dns_names = ["my-service.demeter.run"]
    operator_image_tag = "latest"
    proxy_image_tag = "latest"

    instances = {
      instance1 = {
        image       = "my-instance-image:latest"
        salt        = "my-salt"
        network     = "mainnet"
        utxorpc_url = "http://my-utxorpc-url"
        replicas    = 2
        resources = {
          limits = {
            cpu = "500m"
            memory = "2Gi"
          }
          requests = {
            cpu = "300m"
            memory = "1Gi"
          }
        }
        tolerations = [
          {
            effect = "NoSchedule"
            key = "my-custom-key"
            operator = "Exists"
          }
        ]
      },
      instance2 = {
        image       = "another-instance-image:latest"
        salt        = "another-salt"
        network     = "preprod"
        utxorpc_url = "http://another-utxorpc-url"
      }
    }
    ```

    **Required Variables:**

    * `namespace`: The Kubernetes namespace where the resources will be deployed.
    * `dns_names`: A list of DNS names for the proxy service.
    * `operator_image_tag`: The tag for the feature operator image.
    * `proxy_image_tag`: The tag for the proxy image.
    * `instances`: A map defining the instances to deploy. Each instance requires:
        * `image`: The image for the instance.
        * `salt`: A salt value.
        * `network`: The network the instance belongs to.
        * `utxorpc_url`: The URL for the utxorpc service.

    **Optional Variables:**

    * `dns_zone`: The DNS zone (default: `demeter.run`).
    * `extension_domain`: The extension domain (default: `balius-m1.demeter.run`).
    * `networks`: A list of networks (default: `["mainnet", "preprod", "preview"]`).
    * `metrics_delay`: The metrics delay for the operator (default: `60`).
    * `operator_tolerations`: Tolerations for the operator (default provided in `variables.tf`).
    * `operator_resources`: Resource limits and requests for the operator (default provided in `variables.tf`).
    * `proxy_replicas`: The number of proxy replicas (default: `1`).
    * `proxy_resources`: Resource limits and requests for the proxy (default provided in `variables.tf`).
    * `proxy_tolerations`: Tolerations for the proxy (default provided in `variables.tf`).
    * `instances[].replicas`: The number of replicas for each instance (default: `1`).
    * `instances[].resources`: Resource limits and requests for each instance (default provided in `main.tf`).
    * `instances[].tolerations`: Tolerations for each instance (default provided in `main.tf`).

4.  **Initialize Terraform:**
    ```bash
    terraform init
    ```

5.  **Plan the Deployment:**
    ```bash
    terraform plan
    ```

    Review the plan to ensure it matches your expectations.

6.  **Apply the Configuration:**
    ```bash
    terraform apply
    ```

    Confirm the deployment by typing `yes` when prompted.

7.  **Verify the Deployment:**
    Use `kubectl` to verify that the resources have been deployed successfully.

    ```bash
    kubectl get pods -n <your_namespace>
    kubectl get services -n <your_namespace>
    kubectl get deployments -n <your_namespace>
    ```

## Destroying the Deployment

To destroy the deployed resources, run:

```bash
terraform destroy
