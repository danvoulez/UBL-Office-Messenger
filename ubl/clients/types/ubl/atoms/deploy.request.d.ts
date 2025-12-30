export type DeployStrategy = "canary" | "blue-green" | "rolling";
export type DeployEnv = "dev" | "stg" | "prod";

export interface DeployRequest {
  kind: "deploy/request";
  tenant: string;
  app: string;
  env: DeployEnv;
  image_digest: string;
  strategy: DeployStrategy;
  desired_replicas?: number;
}


