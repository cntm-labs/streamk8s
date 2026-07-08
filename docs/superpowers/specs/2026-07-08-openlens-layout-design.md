# Design: OpenLens-like Logical Grouping for Resource Sidebar

## Architecture & Data Flow
- **Data Source:** The backend continues to send the raw List of `DynamicResourceInfo` grouped by K8s API Group (e.g. `core`, `apps`).
- **Data Transformation (Frontend):** 
  - `ResourceTree.vue` will maintain a static mapping dictionary: `categoryMap`.
  - We define standard K8s categories: **Workloads**, **Network**, **Config**, **Storage**, **Access Control**, and **Custom Resources**.
  - When the raw API resources are fetched, we map them into these logical categories based on their `kind`.
  - Any resource `kind` that does not exist in the `categoryMap` will be dynamically placed into **Custom Resources**, grouped by their original API Group.

## Components Modified
- `src/components/ResourceTree.vue`:
  - `apiGroups` state will be replaced/supplemented by a computed or reactive state `logicalGroups` that shapes the data for rendering.
  - The template will iterate over `logicalGroups` instead of raw API groups.
  - Adds icons and specific sorting for these logical groups.

## Static Map Definition
```javascript
const CATEGORY_MAP = {
  // Workloads
  'Pod': 'Workloads',
  'Deployment': 'Workloads',
  'StatefulSet': 'Workloads',
  'DaemonSet': 'Workloads',
  'ReplicaSet': 'Workloads',
  'Job': 'Workloads',
  'CronJob': 'Workloads',
  // Network
  'Service': 'Network',
  'Endpoints': 'Network',
  'Ingress': 'Network',
  'IngressClass': 'Network',
  'NetworkPolicy': 'Network',
  // Config
  'ConfigMap': 'Config',
  'Secret': 'Config',
  'HorizontalPodAutoscaler': 'Config',
  'PodDisruptionBudget': 'Config',
  // Storage
  'PersistentVolumeClaim': 'Storage',
  'PersistentVolume': 'Storage',
  'StorageClass': 'Storage',
  // Access Control
  'ServiceAccount': 'Access Control',
  'ClusterRole': 'Access Control',
  'ClusterRoleBinding': 'Access Control',
  'Role': 'Access Control',
  'RoleBinding': 'Access Control'
};
```
