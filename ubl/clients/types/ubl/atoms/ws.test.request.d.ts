export interface WsLimits {
  cpu?: number;
  mem_mb?: number;
  timeout_sec?: number;
  net?: boolean;
}

export interface WsTestRequest {
  kind: "ws/test/request";
  tenant: string;
  workspace: string;
  repo: string;
  sha: string;
  suite: string;
  limits?: WsLimits;
}


